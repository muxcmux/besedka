use std::{borrow::Cow, collections::HashMap};

use axum::{
    http::{header::WWW_AUTHENTICATE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::error::DatabaseError;

/// A common error struct for the API which wraps
/// anyhow and sqlx errors and can be converted into
/// an axum response
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Use for 401 responses
    #[error("Authentication required")]
    Unauthorized,
    /// Use for 403 responses
    #[error("You are not allowed to perform this action")]
    Forbidden,
    /// Use for 404 responses
    #[error("Resource not found")]
    NotFound,
    /// Use when request is correct, but contains semantic errors,
    /// e.g. fields fail validation criteria
    #[error("Semantic error in request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },
    /// Wrapper around database errors which returns 404s
    /// when rows can't be found and 500s for anything else
    #[error("{}",
        match .0 {
            sqlx::Error::RowNotFound => "Resource not found",
            _ => "An internal server error occurred"
        }
    )]
    Sqlx(#[from] sqlx::Error),
    /// Any other anyhow errors
    /// this is convenient when used with anyhow!("error")
    /// or when we want to return early from a function
    /// with anyhow::bail!("things crashed")
    /// for this to work, the fn must return an anyhow result
    /// which is then converted into this enum variant
    #[error("An internal server error occured")]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Sqlx(e) => {
                match e {
                    sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }

    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (StatusCode::UNPROCESSABLE_ENTITY, Json(Errors { errors })).into_response();
            },
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    [(WWW_AUTHENTICATE, HeaderValue::from_static("Token"))]
                        .into_iter()
                        .collect::<HeaderMap>(),
                    self.to_string(),
                )
                    .into_response();
            },

            Self::Sqlx(ref e) => {
                tracing::error!("SQLx error: {:?}", e);
            },

            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            },

            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}
pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
