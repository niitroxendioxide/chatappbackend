// Cargo crates
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::Response; // Add Error if necessary later
use tokio_tungstenite::tungstenite::handshake::server::Request;
use std::error;
use serde_json;

// Importing Modules
mod utils;
mod messagehandler;
mod connections;
mod usermanager;
mod uniqueid;

use usermanager::UserManager;
use utils::{json_message, PORT};
use uniqueid::IdGenerator;

/**
 * El protocolo utilizado es WebSocket, el cual permite enviar datos de manera bidireccional y full-duplex
 * 
 * JSON es un lenguaje basado en texto para guardado de datos, así que las requests se pueden enviar como texto y ser JSON
 * Las funciones "Ok" y "Err" son wrappers que representan si está bien o mal el resultado del Error
 * Usar let Err() es como hacer una promise y catchear
*/


#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let listener = TcpListener::bind(PORT).await?;
    println!("WebSocket server listening on ws://{}", PORT);

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

    // la variable _req es porq el linter me ponia q no la usaba entonces la silencie :v
    let callback = |_req: &Request, response: Response| {
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(tcp_stream, callback).await?;
    let (new_user, receiver) = connections::UserConnection::new(IdGenerator::next(), ws_stream).await;
    let user_id = new_user.id;

    // (No necesito saber si el listener se escucha, ya que luego se borra en caso de que no pase)
    new_user.listen(receiver).await;

    UserManager::add_user(new_user).await;

    if let Some(user) = UserManager::get_user(user_id).await {
        let response = serde_json::json!({
            "action": "login",
            "payload": {
                "key":user_id,
                "status":"success",
                "content":"",
            },
            "timestamp":chrono::Utc::now().to_rfc3339(),
        });

        user.send(json_message(response)).await?;

        if let Err(error_msg) = messagehandler::handler::send_history_to_user(&user).await {
            println!("Error sincronizando mensajes al usuario {}, Error: {}", user_id, error_msg)
        };

        if let Err(error_msg) = messagehandler::handler::send_all_usernames(&user).await {
            println!("Error sincronizando mensajes al usuario {}, Error: {}", user_id, error_msg)
        };


        println!("[MAIN]: Connection established, new user: {}", user_id);
    } else {
        println!("[MAIN]: Connection failed after creating user: {}", user_id);
    }

    println!("");

    Ok(())
}