pub mod models;
pub mod auth;
pub mod websocket;
pub mod storage;
pub mod handlers;
pub mod ui;

use sqlx;

use crate::realtime_messenger::auth::AuthError;
use crate::realtime_messenger::websocket::WebSocketError;

pub use self::auth::Auth;
pub use self::handlers::Handlers;
pub use self::storage::Storage;
pub use self::websocket::WebSocketHandler;

#[derive(Debug)]
pub enum MessengerError {
    Auth(AuthError),
    WebSocket(WebSocketError),
    Storage(sqlx::Error),
    Internal(String),
}

#[derive(Clone, Debug)]
pub struct MessengerConfig {
    pub database_url: String,
    pub websocket_addr: String,
    pub jwt_secret: String,
}

pub struct MessengerServer {
    config: MessengerConfig,
}

impl MessengerServer {
    pub async fn new(config: MessengerConfig) -> Result<Self, MessengerError> {
        Ok(Self { config })
    }

    pub async fn run(&self) -> Result<(), MessengerError> {
        Ok(())
    }
}