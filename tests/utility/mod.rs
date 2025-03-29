use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::DatabaseSettings;

pub struct TestApp {
    pub address: String,
    pub pg_pool: PgPool,
}

/// Spin up an instance of the web server and return its address (i.e. http://localhost:XXXXX)
pub async fn spawn_app() -> TestApp {
    let bound_addr = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind address")
        .local_addr()
        .expect("Failed to retrieve local address");
    let address = format!("http://127.0.0.1:{}", bound_addr.port());

    let mut configuration =
        zero2prod::configuration::get_configuration().expect("Failed to read config");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;
    let server = zero2prod::startup::run(&bound_addr, connection_pool.clone())
        .expect("Failed to start server");

    // Tokio spins up a new runtime for each test, shutting down and cleaning up
    // after the test ran. Therefore, no cleanup needed.
    tokio::spawn(server);
    TestApp {
        address,
        pg_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create DB
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate DB
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
