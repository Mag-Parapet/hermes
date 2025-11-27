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
    pub static_host: String,

    pub multer_max_file_size: usize,
    pub default_storage_quota: i64,
    pub default_ssl_enabled: bool,
    pub default_ssl_cert_path: String,
    pub default_ssl_key_path: String,
}

impl Config {
    pub fn init() -> Config {
        dotenv().ok();
        
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let storage_root = env::var("STORAGE_ROOT").unwrap_or_else(|_| "../storage".to_string());
        let static_host = env::var("STATIC_HOST").unwrap_or_else(|_| "/static".to_string());
        let multer_max_file_size = env::var("MULTER_MAX_FILE_SIZE")
            .unwrap_or_else(|_| "104857600".to_string()) // 100 MB default
            .parse::<usize>()
            .expect("MULTER_MAX_FILE_SIZE must be a valid number");
        let default_storage_quota = env::var("DEFAULT_STORAGE_QUOTA")
            .unwrap_or_else(|_| "1073741824".to_string()) 
            .parse::<i64>()
            .expect("DEFAULT_STORAGE_QUOTA must be a valid number");
        let default_ssl_enabled = env::var("DEFAULT_SSL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .expect("DEFAULT_SSL_ENABLED must be a valid boolean");
        let default_ssl_cert_path = env::var("DEFAULT_SSL_CERT_PATH")
            .unwrap_or_else(|_| "/etc/letsencrypt/live/default/fullchain.pem".to_string());
        let default_ssl_key_path = env::var("DEFAULT_SSL_KEY_PATH")
            .unwrap_or_else(|_| "/etc/letsencrypt/live/default/privkey.pem".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");

        Config { 
            jwt_secret,
            port,
            storage_root,
            static_host,
            multer_max_file_size,
            default_storage_quota,
            default_ssl_enabled,
            default_ssl_cert_path,
            default_ssl_key_path
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