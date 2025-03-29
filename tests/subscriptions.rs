mod utility;

use sqlx::{Connection, PgConnection};
use utility::spawn_app;
use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let bound_addr = spawn_app();
    let configuration = get_configuration().expect("Failed to read config");
    let connection_string = configuration.database.connection_string();
    // The `Connection` trait MUST be in scope for us to invoke `PgConnection::connect`
    // - it is not an inherent method of the struct!
    // The connection has to be marked as mutable to query it
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    let endpoint = format!("{bound_addr}/subscriptions");
    let body = "name=frans%20bothma&email=frans%40gmail.com";

    // Act
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved_record = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch the saved subscription.");

    assert_eq!(saved_record.email, "frans@gmail.com");
    assert_eq!(saved_record.name, "frans");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_form_data_is_missing() {
    // Arrange
    let bound_addr = spawn_app();
    let endpoint = format!("{bound_addr}/subscriptions");

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        // Act
        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_msg
        );
    }
}
