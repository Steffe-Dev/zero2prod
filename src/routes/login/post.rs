use actix_web::{HttpResponse, http::header::LOCATION, web};
use secrecy::SecretString;
use sqlx::PgPool;

use crate::authentication::{Credentials, validate_credentials};

use super::error::LoginError;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field:: Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            crate::authentication::AuthError::InvalidCredentials(_) => LoginError::Auth(e.into()),
            crate::authentication::AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish())
}
