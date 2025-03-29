use std::net::TcpListener;

use sqlx::PgConnection;

/// Spin up an instance of the web server and return its address (i.e. http://localhost:XXXXX)
pub fn spawn_app(connection: PgConnection) -> String {
    let bound_addr = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind address")
        .local_addr()
        .expect("Failed to retrieve local address");
    let server = zero2prod::startup::run(&bound_addr, connection).expect("Failed to start server");

    // Tokio spins up a new runtime for each test, shutting down and cleaning up
    // after the test ran. Therefore, no cleanup needed.
    tokio::spawn(server);
    format!("http://127.0.0.1:{}", bound_addr.port())
}
