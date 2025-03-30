use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::configuration;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    zero2prod::telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    let configuration = configuration::get_configuration().expect("Failed to read config");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let bound_addr = TcpListener::bind(address)?.local_addr()?;
    zero2prod::startup::run(&bound_addr, connection)?.await
}
