use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    routes::admin::dashboard::get_username,
    session_state::TypedSession,
    utility::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = match session.get_user_id().map_err(e500)? {
        None => return Ok(see_other("/login")),
        Some(user_id) => user_id,
    };
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }
    let username = get_username(user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password.clone(),
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::Unexpected(error) => Err(e500(error)),
        };
    }
    if form.new_password.expose_secret().len() >= 129 {
        FlashMessage::error("The new password is too long, should be 12 < p < 129.").send();
        return Ok(see_other("/admin/password"));
    }
    if form.new_password.expose_secret().len() <= 12 {
        FlashMessage::error("The new password is too short, should be 12 < p < 129.").send();
        return Ok(see_other("/admin/password"));
    }
    crate::authentication::change_password(&user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}
