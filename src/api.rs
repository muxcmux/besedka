mod error;
pub mod comments;
pub mod extractors;
use std::sync::Arc;

use axum::{Extension, async_trait};
use chrono::{DateTime, Utc};
use ring::hmac;
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;

pub struct AppContext {
    pub db: SqlitePool
}

pub type Context = Extension<Arc<AppContext>>;

pub use error::{Error, ResultExt};

use crate::db::configs::{find_or_default, Config};
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize)]
pub struct Page {
    pub id: i64,
    pub site: String,
    pub path: String,
    pub comments_count: i64,
    pub locked_at: bool,
}

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
struct RequestedConfig {
    site: Option<String>,
    path: Option<String>,
    private: Option<bool>,
    anonymous_comments: Option<bool>,
    moderated: Option<bool>,
    comments_per_page: Option<i64>,
    replies_per_comment: Option<i64>,
    minutes_to_edit: Option<i64>,
    theme: Option<String>,
    user: Option<RequestedUserConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestedUserConfig {
    pub id: String,
    pub username: Option<String>,
    pub name: Option<String>,
    pub moderator: Option<bool>,
    pub avatar: Option<String>
}

#[async_trait]
trait AuthenticatedConfig {
    fn site(&self) -> String;
    fn json(&self) -> Option<String>;
    fn signature(&self) -> Option<String>;

    async fn authenticated_config(&self, db: &SqlitePool,) -> Result<(Config, Option<RequestedUserConfig>)> {
        fn deserialize(json: &Vec<u8>) -> Result<RequestedConfig> {
            let requested: RequestedConfig = serde_json::from_slice(json)?;
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
                .map_err(|_| Error::BadRequest("Cannot verify config object"))?;
            Ok(())
        }

        match self.json() {
            None => Ok((find_or_default(db, &self.site()).await?, None)),
            Some(base64_encoded_json) => {
                match self.signature() {
                    None => Err(Error::BadRequest("Missing signature")),
                    Some(s) => {
                        let json = base64::decode(&base64_encoded_json)
                            .map_err(|_| Error::BadRequest("Can't base64 decode config object"))?;

                        let deserialized = deserialize(&json)?;

                        let site = deserialized.site.unwrap_or(self.site());
                        let config = find_or_default(db, &site).await?;

                        let _ = validate(&json, &s, &config.secret)?;

                        Ok((Config {
                            private:             deserialized.private.unwrap_or(config.private),
                            anonymous_comments:  deserialized.anonymous_comments.unwrap_or(config.anonymous_comments),
                            moderated:           deserialized.moderated.unwrap_or(config.moderated),
                            comments_per_page:   deserialized.comments_per_page.unwrap_or(config.comments_per_page),
                            replies_per_comment: deserialized.replies_per_comment.unwrap_or(config.replies_per_comment),
                            minutes_to_edit:     deserialized.minutes_to_edit.unwrap_or(config.minutes_to_edit),
                            theme:               deserialized.theme.clone().unwrap_or(config.theme),
                            secret:              config.secret,
                            site:                config.site,
                        }, deserialized.user))
                    }
                }
            }
        }
    }
}
