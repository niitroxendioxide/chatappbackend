use std::collections::HashMap;
use std::sync::atomic::{Ordering};
use std::sync::{Mutex};
use std::error::Error;

use super::messages;
use super::usernames::get_user_map;
use super::shared::{MESSAGES, MESSAGE_COUNTER, USER_MAP, UserMessage};
use crate::{usermanager::UserManager, connections::UserConnection, utils::{json_message, ClientMessage}};


pub async fn handle_message(client_msg: &ClientMessage) {
    //println!("[MESSAGE HANDLER]: Request received from User: {}, Action: {}", client_msg.user, client_msg.action);

    if client_msg.action == "msgsend" {
        if let Some(message_content) = client_msg.data.get("message_content") {
            println!("[MESSAGE] User {}: {}", client_msg.user, message_content);
    
            if let Some(replying_to) = client_msg.data.get("replying_to") {
                if let Some(id) = replying_to.as_u64() {
                    send_to_all(&client_msg.user, &message_content.to_string(), id as usize).await;
                } else {
                    eprintln!("Error parsing message value: {:?}", replying_to);
                }
            }
        }
    } else if client_msg.action == "setuser" {
        if let Some(new_username) = client_msg.data.get("message_content") {
            let username_str = new_username.to_string();

            let user_id = match client_msg.user.parse::<usize>() {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Invalid user ID format: {}", client_msg.user);
                    return; // or handle the error as appropriate
                }
            };

            {
                let usernames = USER_MAP.get_or_init(|| Mutex::new(HashMap::new()));
                let mut locked_users = usernames.lock().unwrap();
                locked_users.insert(user_id, username_str.clone());
            }

            println!("[USERNAME CHANGE] User {}: Name: {}", client_msg.user, new_username);
            let network_message = messages::pack_message("set_username", &client_msg.user, &username_str);

            if let Err(e) = UserManager::broadcast(network_message).await {
                eprintln!("Broadcast failed: {}", e);
            }
        }
    }

}


pub async fn send_all_usernames(user: &UserConnection) -> Result<(), Box<dyn Error>> {
    // Collect usernames into a Vec before entering async operations
    let user_list: Vec<(usize, String)> = {
        let usernames = USER_MAP.get_or_init(|| Mutex::new(HashMap::new()));
        let locked_users = usernames.lock().unwrap();
        locked_users.iter()
            .map(|(id, name)| (*id, name.clone()))
            .collect()
    };

    for (id, name) in user_list {
        let id_str = format!("{}", id);
        println!("{}", name);
        let network_message = messages::pack_message("set_username", &id_str, &name);

        if let Err(e) = user.send(network_message).await {
            println!("Error sending username to user: {}", user.id);
        };
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    Ok(())
}

pub async fn send_to_all(user: &str, event: &str, replying_to: usize) {
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
        replying_to: replying_to,
    };

    let network_message = messages::pack_message("message", user, event);

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
                "replying_to": msg.replying_to,
            },
            "timestamp": msg.timestamp
        }));

        user.send(message).await?;
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