#![allow(clippy::async_yields_async)]
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, db_pool))]
// Just adding this param means that we get a 400 when it is missing
pub async fn confirm(
    parameters: web::Query<Parameters>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let id = match get_subscriber_id_from_token(&db_pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError(),
    };

    match id {
        // Non-existing token!
        None => HttpResponse::Unauthorized(),
        Some(subscriber_id) => {
            if confirm_subscriber(&db_pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError();
            }
            HttpResponse::Ok()
        }
    }
}

#[tracing::instrument(
    name = "Get subscriber id from token in the database",
    skip(subscription_token, db_pool)
)]
async fn get_subscriber_id_from_token(
    db_pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let subcriber_id = sqlx::query!(
        r#"
            SELECT subscriber_id
            FROM subscription_tokens
            WHERE subscription_token = $1
        "#,
        subscription_token,
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
