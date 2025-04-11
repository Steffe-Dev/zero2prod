pub mod error;

use actix_web::{HttpResponse, web};
use anyhow::Context;
use anyhow::Ok;
use sqlx::PgPool;

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

#[tracing::instrument("Publishing a newsletter", skip(body, pool))]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, error::PublishError> {
    let subscribers = get_confirmed_subscribers(&pool)
        .await
        .context("Failed to get confirmed subscribers")?;
    for subscriber in subscribers {
        match subscriber {
            core::result::Result::Ok(subscriber) => {
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
            core::result::Result::Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)
}
