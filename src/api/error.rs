use axum::{
    http::StatusCode,
    response::IntoResponse
};

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
    #[error("{}", .0)]
    UnprocessableEntity(&'static str),
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
    #[error("{}", .0)]
    Json(#[from] serde_json::Error),
    #[error("{}", .0)]
    BadRequest(&'static str)
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Json(_) => StatusCode::BAD_REQUEST,
            Self::Sqlx(e) => {
                match e {
                    sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
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
