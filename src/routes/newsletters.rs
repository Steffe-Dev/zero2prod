pub mod error;

use actix_web::HttpRequest;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use base64::Engine;
use error::PublishError;
use secrecy::SecretString;
use sqlx::PgPool;

use crate::authentication::{AuthError, Credentials, validate_credentials};
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;

#[derive(serde::Deserialize, Debug)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize, Debug)]
pub struct Content {
    html: String,
    text: String,
}

#[tracing::instrument(
    name="Publish a newsletter issue", 
    skip(body, pool, email_client, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, error::PublishError> {
    let credentials = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        // We match on `AuthError`'s variants, but we pass the **whole** error
        // into the constructors for `PublishError` variants. This ensures that
        // the context of the top-level wrapper is preserved when the error is
        // logged by our middleware.
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::Auth(e.into()),
            AuthError::Unexpected(_) => PublishError::Unexpected(e.into()),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));
    let subscribers = get_confirmed_subscribers(&pool)
        .await
        .context("Failed to get confirmed subscribers")?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    // `with_context` is lazy, closure is only called on error
                    // Use it if your context has a runtime cost to avoid executing it
                    // in successful cases
                    .with_context(move || {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    // We record the error chain as a structured field
                    // on the log record.
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their email address is invalid",
                );
            }
        }
    }
    Result::Ok(HttpResponse::Ok().finish())
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valed UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    // Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();

    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: SecretString::from(password),
    })
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument("Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
    // We are returning a `Vec` of `Result`s in the happy case.
    // This allows the caller to bubble up errors due to network
    // issues or other transient failures using the `?` operator,
    // while the compiler forces them to handle the subtler
    // mapping error.
    // See http://sled.rs/errors.html for a deep-dive about this technique.
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed';
    "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match r.email.try_into() {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)
}
