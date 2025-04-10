use actix_web::HttpResponse;

#[tracing::instrument("Publishing a newsletter")]
pub async fn publish_newsletter() -> HttpResponse {
    HttpResponse::Ok().finish()
}
