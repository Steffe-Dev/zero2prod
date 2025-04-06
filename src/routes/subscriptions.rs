#![allow(clippy::async_yields_async)]
use actix_web::{
    HttpResponse, Responder,
    web::{self, Form},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{domain::NewSubscriber, email_client::EmailClient};

#[derive(Deserialize)]
pub struct SubscriptionForm {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: Form<SubscriptionForm>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> impl Responder {
    // Implementing a standard library trait for our type conversion makes our intent clear to Rustaceans,
    // so, very ideomatic.
    let new_sub = match form.0.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return HttpResponse::BadRequest(),
    };

    if insert_subscriber(&new_sub, &db_pool).await.is_err() {
        return HttpResponse::InternalServerError();
    }

    if email_client
        .send_email(
            new_sub.email,
            "Welcomen",
            "Welcome to newsletter!",
            "Welcome to newsletter!",
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError();
    }
    HttpResponse::Ok()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_sub, db_pool)
)]
async fn insert_subscriber(new_sub: &NewSubscriber, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_sub.email.as_ref(),
        new_sub.name.as_ref(),
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
