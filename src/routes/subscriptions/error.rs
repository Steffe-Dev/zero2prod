use std::fmt::Formatter;

use actix_web::{ResponseError, http::StatusCode};

#[derive(thiserror::Error)]
pub enum SubscribeError {
    // `{0}` here is like `self.0`
    #[error("{0}")]
    Validation(String),
    // Transparent delegates both `Display`'s and `source`'s implementation
    // to the type wrapped by `UnexpectedError`.
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

// #[derive(thiserror::Error)]
// pub enum SubscribeError {
// // `error` defines the Display representation
// #[error("Failed to acquire a Postgres connection form the pool")]
// // `source` defines the root cause to return from the `Error::source`
// PgPool(#[source] sqlx::Error),
// #[error("Failed to store the confirmation token for a new subscriber.")]
// // `from` automatically derives `From`
// // (e.g. `impl From<StoreTokenError> for SubscribeError {/* */}`)
// // this field is also used as `source`
// StoreToken(#[from] StoreTokenError),
// #[error("Failed to send the confirmation email.")]
// SendEmail(#[from] reqwest::Error),
// #[error("Failed to insert a new subscriber in the database.")]
// InsertSubcriber(#[source] sqlx::Error),
// #[error("Failed to commit SQL transaction to store a new subscriber")]
// TransactionCommit(#[source] sqlx::Error),
//}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeError::Validation(_) => StatusCode::BAD_REQUEST,
            SubscribeError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
