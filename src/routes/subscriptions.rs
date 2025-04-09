#![allow(clippy::async_yields_async)]
use std::fmt::Formatter;

use actix_web::{
    HttpResponse, ResponseError,
    web::{self, Form},
};
use chrono::Utc;
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
) -> Result<HttpResponse, actix_web::Error> {
    // Implementing a standard library trait for our type conversion makes our intent clear to Rustaceans,
    // so, very ideomatic.
    let new_sub = match form.0.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };

    let mut transaction = match db_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let subscriber_id = match insert_subscriber(&new_sub, &mut transaction).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    let subscription_token = SubcriptionToken::generate();
    store_token(&mut transaction, subscriber_id, subscription_token.as_ref()).await?;
    if transaction.commit().await.is_err() {
        return Ok(HttpResponse::InternalServerError().finish());
    }
    if send_confirmation_email(
        email_client,
        new_sub,
        &base_url.0,
        subscription_token.as_ref(),
    )
    .await
    .is_err()
    {
        return Ok(HttpResponse::InternalServerError().finish());
    }
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
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscriber_id, subscription_token) 
            VALUES ($1, $2)
        "#,
        subscriber_id,
        subscription_token
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl ResponseError for StoreTokenError {}

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
            e => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(e)
            }
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
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

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
