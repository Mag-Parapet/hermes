use std::sync::Arc;
use sqlx::PgPool;
use std::env;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub env: Config,
}

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub port: u16,
    pub storage_root: String,
    pub static_host: String, // New Field
}

impl Config {
    pub fn init() -> Config {
        dotenv().ok();
        
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let storage_root = env::var("STORAGE_ROOT").unwrap_or_else(|_| "../storage".to_string());
        let static_host = env::var("STATIC_HOST").unwrap_or_else(|_| "/static".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");

        Config { 
            jwt_secret,
            port,
            storage_root,
            static_host
        }
    }
}

pub async fn connect_db() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}