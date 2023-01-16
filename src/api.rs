mod error;
pub mod comments;
pub mod preview;
pub mod sites;
pub mod pages;
pub mod login;
pub mod extractors;
use std::sync::Arc;

use axum::Extension;
use chrono::{DateTime, Utc};
use ring::{hmac, rand::{SecureRandom, SystemRandom}};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use sqlx::SqlitePool;

use base64::Engine;

pub struct AppContext {
    pub db: SqlitePool
}

pub type Context = Extension<Arc<AppContext>>;

pub use error::Error;

use crate::db::{self, sites::Site, moderators::{Moderator, self}, pages::Page};
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    pub id: i64,
    pub created_at: DateTime<Utc>,
}

impl Cursor {
    fn encode(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&self).unwrap())
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct SignedUser {
    name: Option<String>,
    avatar: Option<String>,
    moderator: Option<bool>,
    op: Option<bool>,
}

struct User {
    name: String,
    moderator: bool,
    op: bool,
    avatar: Option<String>,
}

impl User {
    fn from_moderator(moderator: Moderator) -> Self {
        Self {
            name: moderator.name,
            moderator: true,
            op: moderator.op,
            avatar: moderator.avatar,
        }
    }

    fn from_signed_user(user: SignedUser) -> Self {
        Self {
            name: user.name.unwrap_or(String::from("Anonymous")),
            moderator: user.moderator.unwrap_or(false),
            op: user.op.unwrap_or(false),
            avatar: user.avatar,
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

#[derive(Clone, Debug, sqlx::Type, PartialEq)]
#[sqlx(transparent)]
pub struct Base64(Vec<u8>);

impl Serialize for Base64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let display = base64::display::Base64Display::new(
            &self.0,
            &base64::engine::general_purpose::STANDARD,
        );
        serializer.collect_str(&format!("{}", display))
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
                base64::engine::general_purpose::STANDARD.decode(v)
                    .map(Base64)
                    .map_err(serde::de::Error::custom)
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
        let site = db::sites::find(db, &self.site).await
            .map_err(|_| Error::BadRequest("No configuration found for requested site"))?;

        // logged in moderators always take precedence over 3rd party users
        if let Some(ref sid) = self.sid {
            match moderators::find_by_sid(db, sid).await {
                Err(_) => return Err(Error::Unauthorized),
                Ok(moderator) => return Ok((site, Some(User::from_moderator(moderator)))),
            }
        }

        let user = match &self.user {
            None => None,
            Some(Base64(ref json_bytes)) => match &self.signature {
                None => return Err(Error::BadRequest("Cannot verify user object")),
                Some(Base64(ref s)) => {
                    hmac::verify(&site.key(), json_bytes, s)
                        .map_err(|_| Error::BadRequest("Cannot verify user object"))?;
                    // ok the signature is good
                    let signed_user: SignedUser = serde_json::from_slice(json_bytes)?;
                    if !site.anonymous && signed_user.name.is_none() {
                        return Err(Error::BadRequest("User name is required for non-anonymous sites"))
                    }
                    Some(User::from_signed_user(signed_user))
                }
            }
        };

        Ok((site, user))
    }
}

pub fn generate_random_token() -> Base64 {
    let mut sid = [0_u8; 48];
    let rg = SystemRandom::new();
    let _ = rg.fill(&mut sid);
    Base64(sid.to_vec())
}

fn verify_read_permission(site: &Site, user: &Option<User>, page: Option<&Page>) -> Result<()> {
    if site.private && user.is_none() { return Err(Error::Unauthorized) }

    if let Some(p) = page {
        if p.site != site.site { return Err(Error::BadRequest("Wrong site requested")) }
    }

    Ok(())
}

fn require_moderator(user: &Option<User>) -> Result<()> {
    match user {
        None => return Err(Error::Unauthorized),
        Some(u) => if !u.moderator { return Err(Error::Forbidden) }
    };
    Ok(())
}

#[derive(Serialize)]
struct PageConfig {
    anonymous: bool,
    moderated: bool,
    locked: bool,
}
