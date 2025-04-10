#![allow(clippy::async_yields_async)]
mod error;

use actix_web::{
    HttpResponse,
    web::{self, Form},
};
use anyhow::Context;
use chrono::Utc;
use error::SubscribeError;
use serde::Deserialize;
use sqlx::{Executor, PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    domain::NewSubscriber, domain::SubcriptionToken, email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

#[derive(Deserialize)]
pub struct SubscriptionForm {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: Form<SubscriptionForm>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, error::SubscribeError> {
    // Implementing a standard library trait for our type conversion makes our intent clear to Rustaceans,
    // so, very ideomatic.
    let new_sub = form.0.try_into().map_err(SubscribeError::Validation)?;
    let mut transaction = db_pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection form the pool")?;
    let subscriber_id = insert_subscriber(&new_sub, &mut transaction)
        .await
        .context("Failed to insert a new subscriber in the database.")?;
    let subscription_token = SubcriptionToken::generate();
    store_token(&mut transaction, subscriber_id, subscription_token.as_ref())
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    send_confirmation_email(
        email_client,
        new_sub,
        &base_url.0,
        subscription_token.as_ref(),
    )
    .await
    .context("Failed to send a confirmation email.")?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Store subscription in the database",
    skip(transaction, subscription_token)
)]
async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), error::StoreTokenError> {
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscriber_id, subscription_token) 
            VALUES ($1, $2)
        "#,
        subscriber_id,
        subscription_token
    );
    transaction
        .execute(query)
        .await
        .map_err(error::StoreTokenError)?;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_sub, transaction)
)]
async fn subcriber_exists(
    new_sub: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let query = sqlx::query!(
        r#"
            SELECT id FROM subscriptions WHERE email = $1
        "#,
        new_sub.email.as_ref(),
    );
    match transaction.fetch_one(query).await {
        Ok(row) => Ok(row.get("id")),
        Err(e) => match e {
            sqlx::Error::RowNotFound => Ok(None),
            e => Err(e),
        },
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_sub, transaction)
)]
async fn insert_subscriber(
    new_sub: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let existing_sub = subcriber_exists(new_sub, transaction).await?;

    let subscriber_id = match existing_sub {
        Some(existing_id) => return Ok(existing_id),
        None => Uuid::new_v4(),
    };
    let query = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_sub.email.as_ref(),
        new_sub.name.as_ref(),
        Utc::now()
    );
    transaction.execute(query).await?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send a confirmation link to a new subscriber",
    skip(email_client, new_sub, base_url)
)]
async fn send_confirmation_email(
    email_client: web::Data<EmailClient>,
    new_sub: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let html_body = format!(
        "Welcome to newsletter!<br />\
            Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = format!(
        "Welcome to newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_sub.email, "Welcomen", &html_body, &plain_body)
        .await
}
