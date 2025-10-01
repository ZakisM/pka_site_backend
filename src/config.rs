use std::env;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub yt_api_key: String,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());
        let yt_api_key = env::var("YT_API_KEY").map_err(|_| ConfigError::Missing("YT_API_KEY"))?;
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| default_bind_address());

        Ok(Self {
            database_url,
            redis_url,
            yt_api_key,
            bind_address,
        })
    }
}

fn default_bind_address() -> String {
    "0.0.0.0:1234".to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    Missing(&'static str),
}
