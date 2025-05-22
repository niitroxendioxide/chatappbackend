use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, Mutex};
use serde::Serialize;
use std::error::Error;

use crate::{usermanager::UserManager, connections::UserConnection, utils::{json_message, ClientMessage}};


/*
* Mi linter detesta que uses camelCase para las estructuras o camelCase para las variables, acá se usa
* snake_case para las variables y PascalCase para las estructuras/tipos/implementaciones de tipos
*/
#[derive(Debug, Clone, Serialize)]
struct UserMessage {
    key: usize,
    user: String,
    content: String,
    timestamp: String,
}

/*
* Contadores estáticos
* Son "clases" (acá se llama structs, por que no es en base a objetos, sino a estructuras de datos)
* En el caso de AtomicUsize es un integer de tamaño indefinido y unsigned (sin positivo-negativo, solo natural) & es un contador atómico
* o sea que no puede ser interrumpido y va a ser correcto
* En el caso de messages necesitamos un lock que pertenece a un mutex (vimos esto en vigilante) y que se mutex controle un Vector
* de estructura de datos "UserMessage"
*/
static MESSAGE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static MESSAGES: OnceLock<Mutex<Vec<UserMessage>>> = OnceLock::new();

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

    let network_message = json_message(serde_json::json!({
        "action": "message",
        "payload": {
            "user": user,
            "content": event,
            "key": count,
            "status": "success"
        },
        "timestamp": current_timestamp,
    }));

    let messages = MESSAGES.get_or_init(|| Mutex::new(Vec::new()));
    messages.lock().unwrap().push(saved_message);

    // Envia el mensaje, la parte de "Err" es como el "catch" de otros lenguajes
    if let Err(e) = UserManager::broadcast(network_message).await {
        eprintln!("Broadcast failed: {}", e);
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