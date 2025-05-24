use futures_util::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

pub type Transmitter = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;
pub type Receiver = SplitStream<WebSocketStream<TcpStream>>;

use crate::{messagehandler, usermanager::UserManager, utils::ClientMessage};


#[derive(Clone)]
pub struct UserConnection {
    pub id: usize,
    transmitter: Transmitter,
}

impl UserConnection {
    pub async fn new(id: usize, stream: WebSocketStream<TcpStream>) -> (Self, Receiver) {
        let (transmitter, receiver) = stream.split();
        let mutex_transmitter = Arc::new(Mutex::new(transmitter));

        return (
            UserConnection {
                id,
                transmitter: mutex_transmitter,
            },
            receiver,
        );
    }

    pub async fn listen(&self, receiver: Receiver) {
        let id: usize = self.id;
        let receiver = receiver;

        println!("hola listen");

        tokio::spawn(async move {
            Self::handle_receive(receiver, id).await;
        });
    }

    pub async fn send(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        let mut lock = self.transmitter.lock().await;

        lock.send(msg).await?;

        return Ok(());
    }

    async fn handle_receive(mut receiver: Receiver, user_id: usize) {
        while let Some(msg) = receiver.next().await {
            println!("message received from client");

            match msg {
                Ok(Message::Text(text)) => {
                    println!("mensaje {}", text);

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
