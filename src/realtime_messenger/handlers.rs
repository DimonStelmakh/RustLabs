use warp::{Filter, Rejection, Reply, filters::BoxedFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use super::{
    auth::Auth,
    models::{User, Message},
    storage::Storage,
    websocket::WebSocketHandler
};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    content: String,
    receiver_id: Uuid,
}

#[derive(Serialize)]
pub struct LoginResponse {
    user_id: Uuid,
    user: User,
}

#[derive(Debug)]
pub enum HandlerError {
    Auth(super::auth::AuthError),
    Storage(super::storage::StorageError),
    InvalidInput(String),
}

impl warp::reject::Reject for HandlerError {}

pub struct Handlers {
    auth: Arc<Auth>,
    storage: Arc<Storage>,
    ws_handler: Arc<WebSocketHandler>,
}

#[derive(Deserialize)]
struct WebSocketQuery {
    #[serde(rename = "user-id")]
    user_id: String,
}

#[derive(Deserialize)]
struct MessageQuery {
    limit: i64,
    offset: i64,
}

impl Handlers {
    pub fn new(auth: Auth, storage: Storage, ws_handler: WebSocketHandler) -> Self {
        Self {
            auth: Arc::new(auth),
            storage: Arc::new(storage),
            ws_handler: Arc::new(ws_handler),
        }
    }

    pub fn routes(&self) -> impl Filter<Extract = (Box<dyn Reply>,), Error = Rejection> + Clone {
        let api = self
            .auth_routes()
            .or(self.message_routes())
            .or(self.user_routes()) // New
            .or(self.auth_routes()) // New
            .or(self.ws_routes())
            .recover(Self::handle_rejection);

        warp::path("api")
            .and(api)
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_headers(vec!["content-type", "user-id", "content-length"])
                    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allow_credentials(true)
                    .max_age(3600),
            )
            .map(|reply| Box::new(reply) as Box<dyn Reply>) // Ensure consistent output
    }


    fn auth_routes(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        let login = warp::path!("auth" / "login")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_auth(self.auth.clone()))
            .and_then(Self::handle_login)
            .map(|reply| Box::new(reply) as Box<dyn Reply>);

        let register = warp::path!("auth" / "register")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_auth(self.auth.clone()))
            .and_then(Self::handle_register)
            .map(|reply| Box::new(reply) as Box<dyn Reply>);

        login.or(register).unify().boxed()
    }

    fn user_routes(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path!("users")
            .and(warp::get())
            .and(with_storage(self.storage.clone()))
            .and_then(Self::handle_get_users)
            .boxed()
    }

    fn message_routes(&self) -> BoxedFilter<(impl Reply,)> {
        let get_messages = warp::path!("messages")
            .and(warp::get())
            .and(warp::query::<MessageQuery>())
            .and(warp::header("user-id"))
            .and(with_storage(self.storage.clone()))
            .and_then(Self::handle_get_messages);

        let send_message = warp::path!("messages")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::header("user-id"))
            .and(with_storage(self.storage.clone()))
            .and_then(Self::handle_send_message);

        get_messages.or(send_message).boxed()
    }

    fn ws_routes(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("ws")
            .and(warp::ws())
            .and(warp::query::<WebSocketQuery>())
            .and(with_auth(self.auth.clone()))
            .and(with_ws_handler(self.ws_handler.clone()))
            .and_then(Self::handle_ws_upgrade)
            .boxed()
    }

    async fn handle_login(
        req: LoginRequest,
        auth: Arc<Auth>,
    ) -> Result<impl Reply, Rejection> {
        match auth.login(req.email, req.password).await {
            Ok(user) => Ok(warp::reply::json(&LoginResponse {
                user_id: user.id,
                user,
            })),
            Err(e) => Err(warp::reject::custom(HandlerError::Auth(e))),
        }
    }

    async fn handle_get_users(
        storage: Arc<Storage>,
    ) -> Result<Box<dyn Reply>, Rejection> {
        let users = storage.get_users().await
            .map_err(|e| warp::reject::custom(HandlerError::Storage(e)))?;
        println!("\n");
        println!("users: {:?}", users);
        println!("\n");
        Ok(Box::new(warp::reply::json(&users)))
    }

    async fn handle_register(
        req: RegisterRequest,
        auth: Arc<Auth>,
    ) -> Result<impl Reply, Rejection> {
        match auth.register_user(req.username, req.email, req.password).await {
            Ok(user) => Ok(warp::reply::json(&LoginResponse {
                user_id: user.id,
                user,
            })),
            Err(e) => Err(warp::reject::custom(HandlerError::Auth(e))),
        }
    }

    async fn handle_send_message(
        req: SendMessageRequest,
        user_id: String,
        storage: Arc<Storage>,
    ) -> Result<impl Reply, Rejection> {
        let sender_id = Uuid::parse_str(&user_id)
            .map_err(|_| warp::reject::custom(HandlerError::InvalidInput("Invalid user ID".to_string())))?;

        let message = Message {
            id: Uuid::new_v4(),
            sender_id,
            receiver_id: req.receiver_id,
            content: req.content,
            content_type: super::models::MessageType::Text,
            created_at: chrono::Utc::now(),
            read_at: None,
        };

        match storage.save_message(&message).await {
            Ok(_) => Ok(warp::reply::json(&message)),
            Err(e) => Err(warp::reject::custom(HandlerError::Storage(e))),
        }
    }

    async fn handle_get_messages(
        query: MessageQuery,
        user_id: String,
        storage: Arc<Storage>,
    ) -> Result<impl Reply, Rejection> {
        let user_id = Uuid::parse_str(&user_id)
            .map_err(|_| warp::reject::custom(HandlerError::InvalidInput("Invalid user ID".to_string())))?;

        let messages = storage.get_user_messages(user_id, query.limit, query.offset).await;

        match messages {
            Ok(msgs) => Ok(warp::reply::json(&msgs)),
            Err(e) => Err(warp::reject::custom(HandlerError::Storage(e))),
        }
    }

    async fn handle_ws_upgrade(
        ws: warp::ws::Ws,
        query: WebSocketQuery,
        auth: Arc<Auth>,
        handler: Arc<WebSocketHandler>,
    ) -> Result<impl Reply, Rejection> {
        let user_id = Uuid::parse_str(&query.user_id)
            .map_err(|_| warp::reject::custom(HandlerError::InvalidInput("Invalid user ID".to_string())))?;

        let user = auth.get_user_by_id(user_id).await
            .map_err(|e| warp::reject::custom(HandlerError::Auth(e)))?;

        let user = Arc::new(user);
        let handler = handler.clone();
        Ok(ws.on_upgrade(move |socket| {
            let user = user.clone();
            async move { handler.handle_connection(&user, socket).await }
        }))
    }

    async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
        let (code, message) = if err.is_not_found() {
            (404, "Not Found")
        } else if let Some(e) = err.find::<HandlerError>() {
            match e {
                HandlerError::Auth(_) => (401, "Unauthorized"),
                HandlerError::Storage(_) => (500, "Internal Server Error"),
                HandlerError::InvalidInput(_) => (400, "Bad Request"),
            }
        } else {
            (500, "Internal Server Error")
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": message
            })),
            warp::http::StatusCode::from_u16(code).unwrap(),
        ))
    }
}

fn with_auth(auth: Arc<Auth>) -> impl Filter<Extract = (Arc<Auth>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || auth.clone())
}

fn with_storage(storage: Arc<Storage>) -> impl Filter<Extract = (Arc<Storage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn with_ws_handler(handler: Arc<WebSocketHandler>) -> impl Filter<Extract = (Arc<WebSocketHandler>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || handler.clone())
}