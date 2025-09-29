use std::env;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub rpc_url: String,
    pub websocket: WebSocketConfig,
    pub redis: RedisConfig,
    pub database: DatabaseConfig,
}

#[derive(Clone, Debug)]
pub struct WebSocketConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            rpc_url: env::var("RPC_URL").map_err(|_| ConfigError::MissingEnvVar("RPC_URL"))?,
            websocket: WebSocketConfig {
                host: env::var("WEBSOCKET_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("WEBSOCKET_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|_| ConfigError::InvalidPort("WEBSOCKET_PORT"))?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .map_err(|_| ConfigError::InvalidNumber("DATABASE_MAX_CONNECTIONS"))?,
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),

    #[error("Invalid port configuration for: {0}")]
    InvalidPort(&'static str),

    #[error("Invalid number configuration for: {0}")]
    InvalidNumber(&'static str),
}
