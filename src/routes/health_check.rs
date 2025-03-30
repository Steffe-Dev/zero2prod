use actix_web::{HttpResponse, Responder};
use uuid::Uuid;

pub async fn health_check() -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Performing health check", %request_id);
    let _ = request_span.enter();
    HttpResponse::Ok()
}
