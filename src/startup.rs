use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self},
};
use sqlx::PgPool;
use std::net::SocketAddr;

pub fn run(address: &SocketAddr, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the db_pool in a smart, reference-counted, thread-safe pointer,
    // such that various instances of the app can share the same db connection
    let db_pool = web::Data::new(db_pool);
    // Move the connection into the closure
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(crate::routes::health_check))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
            // Register DB connection as part of application state
            .app_data(web::Data::clone(&db_pool))
    })
    .bind(address)?
    .run();

    Ok(server)
}
