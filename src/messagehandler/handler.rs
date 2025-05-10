use crate::utils::ClientMessage;

pub fn handle_message(client_msg: &ClientMessage) {
    println!("Request received. User: {}, Action: {}", client_msg.user, client_msg.action);

    if let Some(value) = client_msg.data.get("key") {
        println!("Nested value: {}", value);
    }
}