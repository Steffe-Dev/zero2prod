use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self},
};
use std::net::SocketAddr;

pub fn run(address: &SocketAddr) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(crate::routes::health_check))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
    })
    .bind(address)?
    .run();

    Ok(server)
}
