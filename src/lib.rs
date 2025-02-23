use reqwest::{Client, header};
use crate::error::GptError;
use crate::config::GptConfig;
use crate::models::{GptRequest, Message, GptResponse};

pub mod error;
pub mod config;
pub mod models;

pub struct GptClient {
    client: Client,
    api_url: String,
    api_key: String,
    config: GptConfig,
}

impl GptClient {
    pub fn builder() -> GptClientBuilder {
        GptClientBuilder::default()
    }

    pub async fn ask(&self, message: &str) -> Result<String, GptError> {
        tracing::info!("Sending request to GPT API");
        tracing::debug!("Message content length: {}", message.len());

        let headers = self.build_headers()?;
        let body = self.build_request(message);

        tracing::debug!("Making API request to: {}", self.api_url);
        let response = self.client
            .post(&self.api_url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        tracing::debug!("Received response with status: {}", status);

        if !status.is_success() {
            let error_message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("API request failed: {} - {}", status, error_message);
            return Err(GptError::ApiError {
                status_code: status.as_u16(),
                message: error_message,
            });
        }

        let response_data: GptResponse = response.json().await
            .map_err(|e| {
                tracing::error!("Failed to parse API response: {}", e);
                GptError::ParseError(e.to_string())
            })?;

        tracing::info!("Successfully received and parsed API response");
        response_data.choices.first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| {
                tracing::error!("No response choices available in API response");
                GptError::ParseError("No response choices available".to_string())
            })
    }

    fn build_headers(&self) -> Result<header::HeaderMap, GptError> {
        tracing::debug!("Building request headers");
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "api-key",
            header::HeaderValue::from_str(&self.api_key)?
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        Ok(headers)
    }

    fn build_request(&self, message: &str) -> GptRequest {
        tracing::debug!("Building API request body");
        GptRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: self.config.stop.clone(),
        }
    }
}

#[derive(Default)]
pub struct GptClientBuilder {
    api_url: Option<String>,
    api_key: Option<String>,
    config: Option<GptConfig>,
}

impl GptClientBuilder {
    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        tracing::debug!("Setting API URL: {}", url);
        self.api_url = Some(url);
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        tracing::debug!("Setting API key: {}", "*".repeat(key.len()));
        self.api_key = Some(key);
        self
    }

    pub fn config(mut self, config: GptConfig) -> Self {
        tracing::debug!("Setting custom configuration: {:?}", config);
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<GptClient, GptError> {
        tracing::info!("Building GPT client");

        let api_url = self.api_url
            .ok_or_else(|| {
                tracing::error!("API URL is required but not provided");
                GptError::ConfigError("API URL is required".to_string())
            })?;

        let api_key = self.api_key
            .ok_or_else(|| {
                tracing::error!("API key is required but not provided");
                GptError::ConfigError("API key is required".to_string())
            })?;

        Ok(GptClient {
            client: Client::new(),
            api_url,
            api_key,
            config: self.config.unwrap_or_default(),
        })
    }
}