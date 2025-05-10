use tokio_tungstenite::tungstenite::client;

use crate::{usermanager::UserManager, utils::{json_message, ClientMessage}};

pub async fn handle_message(client_msg: &ClientMessage) {
    println!("[MESSAGEHANDLER]: Request received from User: {}, Action: {}", client_msg.user, client_msg.action);

    if let Some(message_content) = client_msg.data.get("message_content") {
        println!("User {} said: {}", client_msg.user, message_content);

        send_to_all(&client_msg.user, &message_content.to_string()).await;
    }
}

pub async fn send_to_all(user: &str, event: &str) {
    let message = json_message(serde_json::json!({
        "action": "message",
        "payload": {
            "user":user,
            "content":event,
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    if let Err(e) = UserManager::broadcast(message).await {
        eprintln!("Broadcast failed: {}", e);
    }
}