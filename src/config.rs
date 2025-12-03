use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub github_webhook_secret: String,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        Ok(Config {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3010".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            database_url: env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?,
            github_webhook_secret: env::var("GITHUB_WEBHOOK_SECRET")
                .map_err(|_| ConfigError::MissingWebhookSecret)?,
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("DATABASE_URL environment variable is required")]
    MissingDatabaseUrl,
    #[error("GITHUB_WEBHOOK_SECRET environment variable is required")]
    MissingWebhookSecret,
    #[error("Invalid PORT value")]
    InvalidPort,
}
