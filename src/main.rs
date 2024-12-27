mod realtime_messenger;

use crate::realtime_messenger::{Auth, Handlers, Storage, WebSocketHandler};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::path::PathBuf;
use std::net::SocketAddr;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    let auth = Auth::new(pool.clone());
    let storage = Storage::new(
        pool.clone(),
        PathBuf::from(env::var("FILE_STORAGE_PATH").expect("FILE_STORAGE_PATH must be set")),
    );
    let ws_handler = WebSocketHandler::new(storage.clone());

    let handlers = Handlers::new(auth, storage, ws_handler);

    let api_routes = handlers.routes();
    let web_routes = realtime_messenger::ui::web::web_routes();
    let routes = api_routes.or(web_routes);

    let addr: SocketAddr = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()
        .expect("Invalid BIND_ADDRESS");

    println!("Server starting on {}", addr);
    warp::serve(routes).run(addr).await;

    Ok(())
}