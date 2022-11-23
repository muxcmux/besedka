mod error;
pub mod comments;
pub mod login;
pub mod extractors;
use std::sync::Arc;

use axum::Extension;
use chrono::{DateTime, Utc};
use ring::hmac;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use sqlx::SqlitePool;

pub struct AppContext {
    pub db: SqlitePool
}

pub type Context = Extension<Arc<AppContext>>;

pub use error::Error;

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
struct User {
    name: String,
    avatar: Option<String>,
    moderator: bool,
}

impl User {
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
    user: Option<Base64>,
    signature: Option<Base64>,
    sid: Option<Base64>,
    payload: Option<T>
}

#[derive(Debug)]
struct Base64(Vec<u8>);
impl Serialize for Base64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&base64::display::Base64Display::with_config(
            &self.0,
            base64::STANDARD,
        ))
    }
}

impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vis;
        impl serde::de::Visitor<'_> for Vis {
            type Value = Base64;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a base64 string")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                base64::decode(v).map(Base64).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(Vis)
    }
}

impl<T> ApiRequest<T> {
    /// Returns the config for the requested site
    /// and an authorised user, either a moderator
    /// by a sid, or a signed 3rd party user
    async fn extract_verified(&self, db: &SqlitePool) -> Result<(Site, Option<User>)> {
        // Fail if there's no config for the requested site
        let site = find(db, &self.site).await
            .map_err(|_| Error::BadRequest("No configuration found for requested site"))?;

        // logged in moderators always take precedence over 3rd party users
        if let Some(Base64(ref sid)) = self.sid {
            match find_by_sid(db, sid).await {
                Err(_) => return Err(Error::Unauthorized),
                Ok(moderator) => return Ok((site, Some(User::from_moderator(moderator)))),
            }
        }

        let user = match &self.user {
            None => None,
            Some(Base64(ref json_bytes)) => match &self.signature {
                None => None,
                Some(Base64(ref s)) => {
                    hmac::verify(&site.key(), json_bytes, s)
                        .map_err(|_| Error::BadRequest("Cannot verify user object"))?;
                    // ok the signature is good
                    Some(serde_json::from_slice(json_bytes)?)
                }
            }
        };

        Ok((site, user))
    }
}
