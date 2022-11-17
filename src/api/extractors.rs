use std::collections::HashMap;
use anyhow::anyhow;
use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response}, Extension, RequestPartsExt
};

use crate::{api::{Error, Result}, db::users::find_moderator_by_sid};

use super::{Cursor, Context};

#[async_trait]
impl<T: Send + Sync> FromRequestParts<T> for Cursor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &T) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        match query.get("cursor") {
            Some(encoded_cursor) => {
                let decoded_cursor = base64::decode(encoded_cursor)
                    .map_err(|err| Error::Anyhow(anyhow!(err)).into_response())?;
                let cursor: Cursor = serde_json::from_slice(&decoded_cursor)
                    .map_err(|err| Error::Anyhow(anyhow!(err)).into_response())?;

                Ok(cursor)
            }
            None => {
                Err((StatusCode::BAD_REQUEST, "`cursor` query param is missing").into_response())
            }
        }
    }
}

use crate::db::users::User;

pub type AuthenticatedModerator = User;

#[async_trait]
impl<T: Send + Sync> FromRequestParts<T> for AuthenticatedModerator {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &T) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        match query.get("sid") {
            Some(sid) => {
                let ctx = parts
                    .extract::<Context>()
                    .await
                    .map_err(|err| err.into_response())?;

                Ok(find_moderator_by_sid(&ctx.db, sid)
                    .await
                    .map_err(|err| Error::Sqlx(err).into_response())?
                )
            }
            None => {
                Err((StatusCode::BAD_REQUEST, "`sid` query param is missing").into_response())
            }
        }
    }
}
