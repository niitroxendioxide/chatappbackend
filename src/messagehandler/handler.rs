use std::sync::atomic::{Ordering};
use std::sync::{Mutex};
use std::error::Error;

use super::messages;
use super::shared::{MESSAGES, MESSAGE_COUNTER, UserMessage};
use crate::{usermanager::UserManager, connections::UserConnection, utils::{json_message, ClientMessage}};


pub async fn handle_message(client_msg: &ClientMessage) {
    println!("[MESSAGEHANDLER]: Request received from User: {}, Action: {}", client_msg.user, client_msg.action);

    if let Some(message_content) = client_msg.data.get("message_content") {
        println!("User {} said: {}", client_msg.user, message_content);

        send_to_all(&client_msg.user, &message_content.to_string()).await;
    }
}

pub async fn send_to_all(user: &str, event: &str) {
    // Contador con la id del mensaje (suma uno)
    // "Ordering::Relaxed" significa que solo utiliza operaciones atómicas y ningun lio raro, o sea, solo suma 1 :v
    let count = MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let current_timestamp = chrono::Utc::now().to_rfc3339();

    // Para tablas de este estilo especificamos que se usa una estructura de datos del tipo 'userMessage'
    let saved_message = UserMessage {
        key: count,
        user: user.to_string(),
        content: event.to_string(),
        timestamp: current_timestamp.clone(),
    };

    let network_message = messages::pack_message(user, event);

    if let Err(e) = UserManager::broadcast(network_message).await {
        eprintln!("Broadcast failed: {}", e);
    } else {
        // guarda mensaje en caso de no haber fallado el broadcast
        let messages = MESSAGES.get_or_init(|| Mutex::new(Vec::new()));
        messages.lock().unwrap().push(saved_message);
    }
}

pub async fn send_history_to_user(user: &UserConnection) -> Result<(), Box<dyn Error>> {
    let messages = MESSAGES.get_or_init(|| Mutex::new(Vec::new()));
    let history = messages.lock().unwrap().clone();

    for msg in history {
        let message = json_message(serde_json::json!({
            "action": "history",
            "payload": {
                "key": msg.key,
                "user": msg.user,
                "content": msg.content,
            },
            "timestamp": msg.timestamp
        }));

        user.send(message).await?;
        // Small delay to prevent flooding (adjust as needed)
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let all_users = UserManager::get_users().lock().await;
    for other_user in all_users.values() {
        if other_user.id == user.id {continue;}

        let sync_old_user_message = json_message(serde_json::json!({
            "action":"user_add",
            "payload":{
                "content":other_user.id,
            },
            "timestamp":chrono::Utc::now().to_rfc3339(),
        }));
        
        if let Err(e) = user.send(sync_old_user_message).await {
            println!("Error sending message to user {}. {}", user.id, e);
        }

        //
        let new_user_added = json_message(serde_json::json!({
            "action":"user_add",
            "payload":{
                "content":user.id,
            },
            "timestamp":chrono::Utc::now().to_rfc3339(),
        }));

        if let Err(e) = other_user.send(new_user_added).await {
            println!("Error sending message to user {}. {}", other_user.id, e);
        }
    }    
    

    Ok(())
}