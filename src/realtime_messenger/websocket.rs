use crate::realtime_messenger::models::{Message, User};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use warp::ws::{Message as WsMessage, WebSocket};
use crate::realtime_messenger::Storage;

#[derive(Debug)]
pub enum WebSocketError {
    ConnectionError,
    MessageSendError,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketCommand {
    SendMessage {
        content: String,
        receiver_id: Uuid,
    },
    MarkAsRead {
        message_ids: Vec<Uuid>,
    },
    Typing {
        receiver_id: Uuid,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketEvent {
    MessageReceived(Message),
    MessageRead {
        message_ids: Vec<Uuid>,
        user_id: Uuid,
    },
    UserTyping {
        user_id: Uuid,
    },
    UserOnline(Uuid),
    UserOffline(Uuid),
}

type Users = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Result<WsMessage, warp::Error>>>>>;

pub struct WebSocketHandler {
    users: Users,
    storage: Arc<Storage>,
}

impl WebSocketHandler {
    pub fn new(storage: Storage) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(storage),
        }
    }

    pub async fn handle_connection(
        &self,
        user: &User,
        ws: WebSocket,
    ) {
        let (ws_sender, mut ws_receiver) = ws.split();
        let (tx, _rx) = mpsc::unbounded_channel();

        self.users.write().await.insert(user.id, tx.clone());

        self.broadcast_user_status(user.id, true).await;

        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        if let Ok(command) = serde_json::from_str::<WebSocketCommand>(text) {
                            self.handle_command(user.id, command).await;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        self.users.write().await.remove(&user.id);
        self.broadcast_user_status(user.id, false).await;
    }

    async fn handle_command(&self, sender_id: Uuid, command: WebSocketCommand) {
        match command {
            WebSocketCommand::SendMessage { content, receiver_id } => {
                println!("Processing message from {} to {}", sender_id, receiver_id);

                let message = Message {
                    id: Uuid::new_v4(),
                    sender_id,
                    receiver_id,
                    content,
                    content_type: crate::realtime_messenger::models::MessageType::Text,
                    created_at: chrono::Utc::now(),
                    read_at: None,
                };

                if let Err(e) = self.storage.save_message(&message).await {
                    eprintln!("Failed to save message: {:?}", e);
                    return;
                }
                println!("Message saved to database");

                self.send_to_user(receiver_id, &WebSocketEvent::MessageReceived(message.clone())).await;
                println!("Message sent to recipient");
            },
            WebSocketCommand::MarkAsRead { message_ids } => {
                if let Some(first_message) = message_ids.first() {
                    if let Some(sender) = self.users.read().await.get(first_message) {
                        let event = WebSocketEvent::MessageRead {
                            message_ids,
                            user_id: sender_id,
                        };
                        let _ = sender.send(Ok(WsMessage::text(serde_json::to_string(&event).unwrap())));
                    }
                }
            }
            WebSocketCommand::Typing { receiver_id } => {
                self.send_to_user(
                    receiver_id,
                    &WebSocketEvent::UserTyping { user_id: sender_id },
                )
                    .await;
            }
        }
    }

    async fn send_to_user(&self, user_id: Uuid, event: &WebSocketEvent) {
        if let Some(sender) = self.users.read().await.get(&user_id) {
            let event_json = serde_json::to_string(&event).unwrap();
            println!("Sending WebSocket message to {}: {}", user_id, event_json);
            let _ = sender.send(Ok(WsMessage::text(event_json)));
        } else {
            println!("User {} not connected to WebSocket", user_id);
        }
    }

    async fn broadcast_user_status(&self, user_id: Uuid, online: bool) {
        let event = if online {
            WebSocketEvent::UserOnline(user_id)
        } else {
            WebSocketEvent::UserOffline(user_id)
        };

        let users = self.users.read().await;
        for (id, sender) in users.iter() {
            if *id != user_id {
                let event_json = serde_json::to_string(&event).unwrap();
                let _ = sender.send(Ok(WsMessage::text(event_json)));
            }
        }
    }
}