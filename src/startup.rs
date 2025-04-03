use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use actix_web::{
    App, HttpServer,
    dev::Server,
    web::{self},
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::io::Error;
use std::net::SocketAddr;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, Error> {
        let db_pool = get_connection_pool(&configuration.database);
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration
                .email_client
                .sender()
                .expect("Failed to parse sender email"),
            configuration.email_client.base_url,
            configuration.email_client.authorisation_token,
            timeout,
        );
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let bound_addr = TcpListener::bind(address)?.local_addr()?;
        let server = run(&bound_addr, db_pool, email_client)?;

        Ok(Self {
            server,
            port: bound_addr.port(),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
}

fn run(
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
