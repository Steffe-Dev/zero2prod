use actix_web::{ResponseError, http::StatusCode};
use std::fmt::Formatter;

use crate::utility::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("{0}")]
    Validation(String),
    #[error("User provided a subcription token that does not belong to them.")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ConfirmError::Validation(_) => StatusCode::BAD_REQUEST,
            ConfirmError::Unauthorized => StatusCode::UNAUTHORIZED,
            ConfirmError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
