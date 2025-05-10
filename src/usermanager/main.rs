// users_manager.rs
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use crate::connections::UserConnection; // Import from parent module

static USERS: OnceLock<Mutex<HashMap<usize, UserConnection>>> = OnceLock::new();

pub struct UserManager;

impl UserManager {
    fn get_users() -> &'static Mutex<HashMap<usize, UserConnection>> {
        USERS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub async fn add_user(user: UserConnection) {
        let users = Self::get_users();
        users.lock().await.insert(user.id, user);
    }

    pub async fn get_user(user_id: usize) -> Option<UserConnection> {
        let users = Self::get_users();
        users.lock().await.get(&user_id).cloned()
    }

    pub async fn remove_user(user_id: usize) {
        let users = Self::get_users();
        users.lock().await.remove(&user_id);

        println!("[USERMANAGER]: User id:[{}] disconnected", user_id)
    }

    pub async fn broadcast(message: Message) -> Result<(), Box<dyn std::error::Error>> {
        let users = Self::get_users().lock().await;

        for user in users.values() {
            if let Err(e) = user.send(message.clone()).await {
                eprintln!("Failed to send to user {}: {}", user.id, e);
            }
        }

        Ok(())
    }
}