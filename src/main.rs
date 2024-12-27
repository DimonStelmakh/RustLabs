mod realtime_messenger;

use crate::realtime_messenger::{Auth, Handlers, Storage, WebSocketHandler};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::path::PathBuf;
use std::net::SocketAddr;
use warp::Filter;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let exe_dir = env::current_exe()?
        .parent()
        .ok_or_else(|| Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not get executable directory",
        )))?
        .to_path_buf();

    let mut storage_dir = exe_dir.join("storage").join("files");

    println!("Current executable directory: {:?}", exe_dir);
    println!("Attempting to create storage at: {:?}", storage_dir);

    match fs::create_dir_all(&storage_dir) {
        Ok(_) => println!("Successfully created storage directory at: {:?}", storage_dir),
        Err(e) => {
            println!("Error creating storage directory: {:?}", e);
            println!("Attempting to create in current directory instead...");

            let fallback_dir = PathBuf::from("storage/files");
            fs::create_dir_all(&fallback_dir)?;
            println!("Successfully created storage directory in current path: {:?}", fallback_dir);
            storage_dir = fallback_dir;
        }
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let auth = Auth::new(pool.clone());
    let storage = Storage::new(pool.clone(), storage_dir);

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