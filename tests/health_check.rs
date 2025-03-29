mod utility;

use sqlx::{Connection, PgConnection};
use utility::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let configuration =
        zero2prod::configuration::get_configuration().expect("Failed to read config");
    let connection_string = configuration.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let bound_addr = spawn_app(connection);
    let endpoint = format!("{bound_addr}/health_check");

    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(endpoint)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
