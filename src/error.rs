use thiserror::Error;

#[derive(Error, Debug)]
pub enum GptError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid header value: {0}")]
    HeaderError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("API response error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}