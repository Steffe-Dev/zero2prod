use crate::utility::error_chain_fmt;
use actix_web::ResponseError;
use actix_web::http::StatusCode;
use std::fmt::Formatter;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            PublishError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
