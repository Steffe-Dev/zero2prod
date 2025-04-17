use actix_web::{HttpResponse, error::InternalError, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::SecretString;
use sqlx::PgPool;

use crate::{
    authentication::{Credentials, validate_credentials},
    session_state::TypedSession,
    utility::see_other,
};

use super::error::LoginError;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, pool, session),
    fields(username=tracing::field:: Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));

            // Rotates the session token when a user logs in (security)
            session.renew();
            // Serialisation of the value might fail
            session
                .insert_user_id(user_id)
                .map_err(|e| login_redirect(LoginError::Unexpected(e.into())))?;
            Ok(see_other("/admin/dashboard"))
        }
        Err(e) => {
            let e = match e {
                crate::authentication::AuthError::InvalidCredentials(_) => {
                    LoginError::Auth(e.into())
                }
                crate::authentication::AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };
            Err(login_redirect(e))
        }
    }
}

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = see_other("/login");
    InternalError::from_response(e, response)
}
