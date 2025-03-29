use actix_web::{App, HttpResponse, HttpServer, Responder, dev::Server, web};
use std::net::SocketAddr;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(address: &SocketAddr) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind(address)?
        .run();

    Ok(server)
}
