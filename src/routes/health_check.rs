#![allow(clippy::async_yields_async)]
use actix_web::{HttpResponse, Responder};

#[tracing::instrument(name = "Performing heatlth check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
