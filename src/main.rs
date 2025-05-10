// Cargo crates
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{handshake::server::Response, Message}; // Add Error if necessary later
use tokio_tungstenite::tungstenite::handshake::server::Request;
//use tokio_tungstenite::WebSocketStream;
use futures_util::{SinkExt, StreamExt};
use std::error;
use serde_json;

// Importing Modules
mod utils;
use utils::ClientMessage;

mod messagehandler;


/**
 * JSON es un lenguaje basado en texto para guardado de datos, así que las requests se pueden enviar como texto y ser JSON
 * Las funciones "Ok" y "Err" son wrappers que representan si está bien o mal el resultado del Error
 * Usar let Err() es como hacer una promise y catchear
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://localhost:8080");

    while let Ok((tcp_stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(tcp_stream).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(tcp_stream: tokio::net::TcpStream) -> Result<(), Box<dyn error::Error>> {
    let callback = |req: &Request, response: Response| {
        println!("Received handshake request: {:?}", req.headers());
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(tcp_stream, callback).await?;
    let (mut tx, mut rx) = ws_stream.split();

    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Entra si recibe un mensaje de tipo texto (JSON es texto)
                println!("Received message: {}", text.to_string());

                match serde_json::from_str::<ClientMessage>(&text) {
                    // SI el mensaje cumple con el tipo de JSON entonces entra al Ok
                    Ok(client_msg) => {
                        messagehandler::handle_message(&client_msg);

                        let response = serde_json::json!({
                            "status": "success",
                            "received_action": client_msg.action
                        });
                        tx.send(Message::Text(response.to_string().into())).await?;
                    }

                    Err(e) => {
                        eprintln!("JSON parse error: {}", e);


                        tx.send(Message::Text(
                            serde_json::json!({ "error": "Invalid JSON" }).to_string().into()
                        )).await?;
                    }
                }
            }

            // Desconectarse
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
                break;
            }

            // En caso de error en el server
            Err(e) => {
                eprintln!("Receive error: {}", e);
                break;
            }
            _ => {} // Otros tipos (case else basciamente)
        }
    }

    Ok(())
}