use std::fmt::Formatter;

use actix_web::{ResponseError, http::StatusCode};

pub enum SubscribeError {
    Validation(String),
    StoreToken(StoreTokenError),
    SendEmail(reqwest::Error),
    PgPool(sqlx::Error),
    InsertSubcriber(sqlx::Error),
    TransactionCommit(sqlx::Error),
}

impl From<reqwest::Error> for SubscribeError {
    fn from(value: reqwest::Error) -> Self {
        Self::SendEmail(value)
    }
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        Self::Validation(value)
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(value: StoreTokenError) -> Self {
        Self::StoreToken(value)
    }
}

impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscribeError::Validation(e) => write!(f, "{}", e),
            SubscribeError::StoreToken(_) => write!(
                f,
                "Failed to store the comfirmation token for a new subscriber.",
            ),
            SubscribeError::SendEmail(_) => write!(f, "Failed to send a confirmation email."),
            SubscribeError::PgPool(_) => {
                write!(f, "Failed to acquire a Postgres connection from the pool.",)
            }
            SubscribeError::InsertSubcriber(_) => {
                write!(f, "Failed to insert a new subscriber in the database.",)
            }
            SubscribeError::TransactionCommit(_) => write!(
                f,
                "Failed to commit SQL transaction to store a new subscriber.",
            ),
        }
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for SubscribeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SubscribeError::Validation(_) => None,
            SubscribeError::StoreToken(store_token_error) => Some(store_token_error),
            SubscribeError::SendEmail(error) => Some(error),
            SubscribeError::PgPool(error) => Some(error),
            SubscribeError::InsertSubcriber(error) => Some(error),
            SubscribeError::TransactionCommit(error) => Some(error),
        }
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeError::Validation(_) => StatusCode::BAD_REQUEST,
            SubscribeError::StoreToken(_)
            | SubscribeError::SendEmail(_)
            | SubscribeError::PgPool(_)
            | SubscribeError::InsertSubcriber(_)
            | SubscribeError::TransactionCommit(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub struct StoreTokenError(pub sqlx::Error);

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The compiler transparently casts `&sqlx::Error` into a `&dyn Error`
        Some(&self.0)
    }
}

fn error_chain_fmt(e: &impl std::error::Error, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}/n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
