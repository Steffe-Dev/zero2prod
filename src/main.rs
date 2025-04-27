use std::fmt::{Debug, Display};

use tokio::task::JoinError;
use zero2prod::configuration;
use zero2prod::issue_delivery_worker::run_worker_until_stopped;
use zero2prod::startup::{self};
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);
    let configuration = configuration::get_configuration().expect("Failed to read config");
    let app = startup::Application::build(configuration.clone()).await?;
    let app_task = tokio::spawn(app.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = app_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o)
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed to complete",
                task_name
            )
        }
    }
}
