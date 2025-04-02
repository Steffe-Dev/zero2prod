use crate::email_client::EmailClient;
use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self},
};
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_actix_web::TracingLogger;

pub fn run(
    address: &SocketAddr,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Wrap the db_pool in a smart, reference-counted, thread-safe pointer,
    // such that various instances of the app can share the same db connection
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    // Move the connection into the closure
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(crate::routes::health_check))
            .route("/subscriptions", web::post().to(crate::routes::subscribe))
            // Register DB connection as part of application state
            .app_data(web::Data::clone(&db_pool))
            .app_data(web::Data::clone(&email_client))
    })
    .bind(address)?
    .run();

    Ok(server)
}
