use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Transmitter = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;
pub type Receiver = futures_util::stream::SplitStream<WebSocketStream<TcpStream>>;

use crate::utils::ClientMessage;
use crate::messagehandler;
use crate::usermanager::UserManager;

#[derive(Clone)]
pub struct UserConnection {
    pub id: usize,
    transmitter: Transmitter,
}

impl UserConnection {
    pub async fn new(id: usize, stream: WebSocketStream<TcpStream>) -> Self {
        let (splittransmitter, receiver) = stream.split();
        let transmitter = Arc::new(Mutex::new(splittransmitter));

        //let transmitter_clone = transmitter.clone();

        tokio::spawn(async move {
            Self::handle_receive(receiver, id).await;
        });

        UserConnection {
            id,
            transmitter,
        }
    }

    pub async fn send(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        let mut lock = self.transmitter.lock().await;

        lock.send(msg).await?;

        return Ok(());
    }

    async fn handle_receive(mut receiver: Receiver, user_id: usize) {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Mensaje aprobado :v
                    match serde_json::from_str::<ClientMessage>(&text) {
                    // SI el mensaje cumple con el tipo de JSON entonces entra al Ok
                        Ok(client_msg) => {
                            messagehandler::handle_message(&client_msg).await;
                        }

                        Err(e) => {
                            eprintln!("JSON parse error: {}", e);
                        }
                    }
                }

                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        UserManager::remove_user(user_id).await;
    }
}