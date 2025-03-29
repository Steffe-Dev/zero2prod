use actix_web::{Responder, web::Form};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SubscriptionForm {
    name: String,
    email: String,
}

pub async fn subscribe(form: Form<SubscriptionForm>) -> impl Responder {
    // HttpResponse::Ok()
    format!("Hello {}, with email {}", form.name, form.email)
}
