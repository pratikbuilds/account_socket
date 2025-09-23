use std::env;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub rpc_url: String,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            rpc_url: env::var("RPC_URL").map_err(|_| ConfigError::MissingEnvVar("RPC_URL"))?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),
}
