use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{stream::StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("WebSocket server listening on ws://localhost:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("WebSocket handshake failed");

    let (mut tx, mut rx) = ws_stream.split();
    println!("Client connected");

    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
                // Fixed line below
                tx.send(Message::Text(format!("Echo: {}", text).into())).await.unwrap();
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}