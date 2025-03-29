use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let bound_addr = TcpListener::bind("127.0.0.1:8000")?.local_addr()?;
    zero2prod::startup::run(&bound_addr)?.await
}
