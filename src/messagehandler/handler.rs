// Add to messagehandler.rs
use std::sync::atomic::{AtomicUsize, Ordering};

static MESSAGE_COUNTER: AtomicUsize = AtomicUsize::new(0);

use crate::{usermanager::UserManager, utils::{json_message, ClientMessage}};

pub async fn handle_message(client_msg: &ClientMessage) {
    println!("[MESSAGEHANDLER]: Request received from User: {}, Action: {}", client_msg.user, client_msg.action);

    if let Some(message_content) = client_msg.data.get("message_content") {
        println!("User {} said: {}", client_msg.user, message_content);

        send_to_all(&client_msg.user, &message_content.to_string()).await;
    }
}

pub async fn send_to_all(user: &str, event: &str) {
    // Increment counter and get current value
    let count = MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed);

    let message = json_message(serde_json::json!({
        "action": "message",
        "payload": {
            "user": user,
            "content": event,
            "key": count,  // Add counter to payload
            "status": "success"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    if let Err(e) = UserManager::broadcast(message).await {
        eprintln!("Broadcast failed: {}", e);
    }
}