use std::sync::atomic::Ordering;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use crate::{utils::json_message};
use chrono;

use super::shared::MESSAGE_COUNTER;

pub fn pack_message(msgtype: &str, user_id: &str, message_content: &str) -> Message {
    
    let network_message = json!({
        "action": msgtype,
        "payload": {
            "user": user_id,
            "content": message_content,
            "status": "success",
            "key": MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    return json_message(network_message);
}