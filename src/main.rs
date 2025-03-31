use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::configuration;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);
    let configuration = configuration::get_configuration().expect("Failed to read config");
    let connection = PgPool::connect_lazy_with(configuration.database.with_db());
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let bound_addr = TcpListener::bind(address)?.local_addr()?;
    zero2prod::startup::run(&bound_addr, connection)?.await
}
