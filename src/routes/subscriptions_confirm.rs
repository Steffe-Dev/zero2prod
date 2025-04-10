#![allow(clippy::async_yields_async)]
mod error;

use actix_web::{HttpResponse, web};
use anyhow::Context;
use error::ConfirmError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::SubcriptionToken;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, db_pool))]
// Just adding this param means that we get a 400 when it is missing
pub async fn confirm(
    parameters: web::Query<Parameters>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmError> {
    let subscripion_token = parameters
        .subscription_token
        .to_owned()
        .try_into()
        .map_err(ConfirmError::Validation)?;
    let id = get_subscriber_id_from_token(&db_pool, subscripion_token)
        .await
        .context("Failed to get a subscriber in the database with the provided token.")?;

    match id {
        // Non-existing token!
        None => return Err(ConfirmError::Unauthorized),
        Some(subscriber_id) => confirm_subscriber(&db_pool, subscriber_id)
            .await
            .context("Failed to confirm the subcriber in the database.")?,
    }
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Get subscriber id from token in the database",
    skip(subscription_token, db_pool)
)]
async fn get_subscriber_id_from_token(
    db_pool: &PgPool,
    subscription_token: SubcriptionToken,
) -> Result<Option<Uuid>, sqlx::Error> {
    let subcriber_id = sqlx::query!(
        r#"
            SELECT subscriber_id
            FROM subscription_tokens
            WHERE subscription_token = $1
        "#,
        subscription_token.as_ref(),
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(subcriber_id.map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed in the database",
    skip(subscriber_id, db_pool)
)]
async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE subscriptions 
            SET status = 'confirmed'
            WHERE id = $1 
        "#,
        subscriber_id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
