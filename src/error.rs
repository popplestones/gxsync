use thiserror::Error;

#[derive(Error, Debug)]
pub enum GxsyncError {
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("State error: {0}")]
    State(#[from] anyhow::Error),

    #[error("Other error: {0}")]
    Other(String),
}
