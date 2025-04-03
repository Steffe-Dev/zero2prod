use zero2prod::configuration;
use zero2prod::startup::{self};
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);
    let configuration = configuration::get_configuration().expect("Failed to read config");
    let app = startup::Application::build(configuration).await?;
    app.run_until_stopped().await?;
    Ok(())
}
