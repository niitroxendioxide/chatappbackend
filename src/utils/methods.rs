use serde::Deserialize;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Deserialize)]
pub struct ClientMessage {
    pub user: String,
    pub action: String,
    pub data: serde_json::Value,
}

pub fn json_message(value: serde_json::Value) -> Message {
    Message::Text(value.to_string().into())
}

pub fn _json_message_generic<T: Serialize>(data: T) -> Result<Message, serde_json::Error> {
    let json_string = serde_json::to_string(&data)?;
    Ok(Message::Text(json_string.into()))
}