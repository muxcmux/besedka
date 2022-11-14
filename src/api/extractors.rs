use std::collections::HashMap;
use anyhow::anyhow;
use axum::{
    async_trait,
    extract::{FromRequestParts, Path, Query},
    http::{request::Parts, Uri, StatusCode},
    response::{IntoResponse, Response}, RequestPartsExt,
};
use sqlx::query_as;

use crate::{
    api::{Context, Error, Result},
    db::configs::{Config, find_or_default},
};

use super::{Page, Cursor};


pub struct ExtractSitePageAndConfig {
    pub site: String,
    pub page: Option<Page>,
    pub config: Config,
}

#[async_trait]
impl<T: Send + Sync> FromRequestParts<T> for ExtractSitePageAndConfig {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &T) -> Result<Self, Self::Rejection> {
        fn site_from_page(page: &str) -> Result<String> {
            let uri: anyhow::Result<Uri, _> = format!("https://{}", page).parse();
            match uri {
                Err(_) => Err(Error::NotFound),
                Ok(uri) => match uri.host() {
                    Some(site) => Ok(site.to_string()),
                    None => Err(Error::NotFound),
                },
            }
        }

        let Path(page) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        let ctx = parts
            .extract::<Context>()
            .await
            .map_err(|err| err.into_response())?;

        let site = site_from_page(&page).map_err(|err| err.into_response())?;

        let config = find_or_default(&ctx.db, &site)
            .await
            .map_err(|err| Error::Anyhow(err).into_response())?;

        let page = query_as!(Page, "SELECT * FROM pages WHERE (site || path = ?)", page)
            .fetch_optional(&ctx.db)
            .await
            .map_err(|err| Error::Sqlx(err).into_response())?;

        Ok(ExtractSitePageAndConfig { site, page, config })
    }
}

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
