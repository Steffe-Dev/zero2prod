use std::net::TcpListener;

/// Spin up an instance of the web server and return its address (i.e. http://localhost:XXXXX)
fn spawn_app() -> String {
    let bound_addr = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind address")
        .local_addr()
        .expect("Failed to retrieve local address");
    let server = zero2prod::run(&bound_addr).expect("Failed to start server");

    // Tokio spins up a new runtime for each test, shutting down and cleaning up
    // after the test ran. Therefore, no cleanup needed.
    tokio::spawn(server);
    format!("http://127.0.0.1:{}", bound_addr.port())
}

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
