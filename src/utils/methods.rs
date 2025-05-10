use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientMessage {
    pub user: String,
    pub action: String,
    pub data: serde_json::Value,
}