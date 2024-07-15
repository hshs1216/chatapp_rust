mod handler;

use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;
use handler::handle_connection;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("Listening on: {}", addr);

    let clients = Arc::new(Mutex::new(Vec::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let clients = Arc::clone(&clients);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, clients).await {
                        eprintln!("Error handling connection: {:?}", e);
                    }
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {:?}", e),
        }
    }
}
