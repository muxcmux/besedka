use anyhow::anyhow;
use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;

use crate::api::{Error, Result};

use super::Cursor;

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
