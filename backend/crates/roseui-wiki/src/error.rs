#![allow(clippy::disallowed_types)]

use roseui_common::ApiError;

/// Wiki domain error. Mapped to the shared [`ApiError`] boundary so handlers
/// can return `Result<_, ApiError>` uniformly.
#[derive(Debug, thiserror::Error)]
pub enum WikiError {
    #[error("wiki page not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<WikiError> for ApiError {
    fn from(err: WikiError) -> Self {
        match err {
            WikiError::NotFound(message) => ApiError::NotFound(message),
            WikiError::BadRequest(message) => ApiError::BadRequest(message),
            WikiError::Internal(message) => ApiError::Internal(message),
        }
    }
}
