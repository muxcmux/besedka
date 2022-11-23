mod error;
pub mod comments;
pub mod login;
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

use crate::db::{sites::{find, Site}, moderators::{Moderator, find_by_sid}};
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
struct Commenter {
    name: String,
    avatar: Option<String>,
    moderator: bool,
}

impl Commenter {
    fn from_moderator(moderator: Moderator) -> Self {
        Self {
            name: moderator.name,
            avatar: moderator.avatar,
            moderator: true,
        }
    }
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
    fn sid(&self) -> Result<Vec<u8>> {
        match self.sid {
            None => Err(Error::BadRequest("No sid found in request")),
            Some(ref s) => base64::decode(s)
                .map_err(|_| Error::BadRequest("Can't base64 decode signature"))
        }
    }
    /// Returns the config for the requested site
    /// and an authorised user, either a moderator
    /// by a sid, or a signed 3rd party user
    async fn extract_verified(&self, db: &SqlitePool) -> Result<(Site, Option<Commenter>)> {
        fn deserialize(json_bytes: &Vec<u8>) -> Result<Commenter> {
            let requested: Commenter = serde_json::from_slice(json_bytes)?;
            Ok(requested)
        }

        fn validate(
            json_bytes: &Vec<u8>,
            signature: &String,
            secret: &Vec<u8>,
        ) -> Result<()>
        {
            let decoded = base64::decode(signature)
                .map_err(|_| Error::BadRequest("Can't base64 decode signature"))?;
            let key = hmac::Key::new(hmac::HMAC_SHA256, &secret);
            hmac::verify(&key, json_bytes, &decoded)
                .map_err(|_| Error::BadRequest("Cannot verify user object"))?;
            Ok(())
        }

        // Fail if there's no config for the requested site
        let site = match find(db, &self.site).await? {
            None => return Err(Error::BadRequest("No configuration found for requested site")),
            Some(s) => s,
        };

        // logged in moderators always take precedence over 3rd party users
        if let Ok(sid) = self.sid() {
            match find_by_sid(db, &sid).await {
                Err(_) => return Err(Error::Unauthorized),
                Ok(moderator) => return Ok((site, Some(Commenter::from_moderator(moderator)))),
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
                    let _ = validate(&json, s, &site.secret)?;

                    // ok the signature is good
                    Some(deserialized_user)
                }
            }
        };

        Ok((site, user))
    }
}
