mod utility;

use utility::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let bound_addr = spawn_app();
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
