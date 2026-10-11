use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub yt_api_key: String,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default)]
    pub expose_openapi: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        envy::from_env().map_err(ConfigError::Load)
    }
}

fn default_bind_address() -> String {
    "0.0.0.0:1234".to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(#[from] envy::Error),
}
