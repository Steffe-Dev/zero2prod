mod utility;

use utility::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let bound_addr = spawn_app();
    let endpoint = format!("{bound_addr}/subscriptions");
    let body = "name=frans%20bothma&email=frans%40gmail.com";

    let client = reqwest::Client::new();

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
