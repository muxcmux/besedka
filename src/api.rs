mod error;
pub mod comments;
pub mod extractors;
use std::sync::Arc;

use axum::Extension;
use chrono::{DateTime, Utc};
use ring::hmac;
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;

pub struct AppContext {
    pub db: SqlitePool
}

pub type Context = Extension<Arc<AppContext>>;

pub use error::{Error, ResultExt};

use crate::db::{configs::{find_or_default, Config}, moderators::{Moderator, find_by_sid}, pages::{find_by_site_and_path, Page}};
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    pub id: i64,
    pub created_at: DateTime<Utc>,
}

impl Cursor {
    fn encode(&self) -> String {
        base64::encode(serde_json::to_vec(&self).unwrap())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ForeignUser {
    pub name: String,
    pub moderator: Option<bool>,
    pub avatar: Option<String>
}

#[derive(Deserialize)]
struct ApiRequest<T> {
    site: String,
    path: String,
    user: Option<String>,
    signature: Option<String>,
    sid: Option<String>,
    payload: Option<T>
}

impl<T> ApiRequest<T> {
    /// Returns the config for the requested site
    /// and an authorised user, either a moderator
    /// by a sid, or a signed 3rd party user
    async fn extract(&self, db: &SqlitePool) -> Result<(Config, Option<User>)> {
        fn deserialize(json: &Vec<u8>) -> Result<ForeignUser> {
            let requested: ForeignUser = serde_json::from_slice(json)?;
            Ok(requested)
        }

        fn validate(
            json: &Vec<u8>,
            signature: &String,
            secret: &Vec<u8>,
        ) -> Result<()>
        {
            let decoded = base64::decode(signature)
                .map_err(|_| Error::BadRequest("Can't base64 decode signature"))?;
            let key = hmac::Key::new(hmac::HMAC_SHA256, &secret);
            hmac::verify(&key, json, &decoded)
                .map_err(|_| Error::BadRequest("Cannot verify user object"))?;
            Ok(())
        }

        let config = find_or_default(db, &self.site).await?;

        // logged in moderators always take precedence over 3rd party users
        if let Some(ref sid) = self.sid {
            if let Some(moderator) = find_moderator_by_sid(db, sid).await? {
                return Ok((config, Some(moderator)));
            }
        }

        let user = match &self.user {
            None => None,
            Some(base64_encoded_json) => match &self.signature {
                None => None,
                Some(s) => {
                    let json = base64::decode(&base64_encoded_json)
                        .map_err(|_| Error::BadRequest("Can't base64 decode user object"))?;

                    let deserialized_user = deserialize(&json)?;

                    // failed verification vs the signature will
                    // return an error early
                    let _ = validate(&json, s, &config.secret)?;

                    // ok the signature is good
                    Some(create_or_find_foreign_user(db, &self.site, &deserialized_user).await?)
                }
            }
        };

        Ok((config, user))
    }
}
