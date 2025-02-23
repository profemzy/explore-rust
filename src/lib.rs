use reqwest::{Client, header};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Declare our modules
pub mod error;
pub mod config;
pub mod models;

// Import types from our modules
use crate::error::GptError;
use crate::config::GptConfig;

use crate::models::{
    GptRequest,
    Message,
    GptResponse,
};


// Define our main client structure
pub struct GptClient {
    client: Client,
    api_url: String,
    api_key: String,
    config: GptConfig,
}

// Implement the core functionality
impl GptClient {
    // Constructor using builder pattern
    pub fn builder() -> GptClientBuilder {
        GptClientBuilder::default()
    }

    // Helper method to build headers
    fn build_headers(&self) -> Result<header::HeaderMap, GptError> {
        let mut headers = header::HeaderMap::new();

        // Add the API key header
        headers.insert(
            "api-key",
            header::HeaderValue::from_str(&self.api_key)?
        );

        // Add content type for JSON
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        Ok(headers)
    }

    // Helper method to build the request body
    fn build_request(&self, message: &str) -> GptRequest {
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
            stream: false,
        }
    }

    // Method for regular (non-streaming) requests
    pub async fn ask(&self, message: &str) -> Result<String, GptError> {
        tracing::info!("Sending request to GPT API");
        tracing::debug!("Message content length: {}", message.len());

        let headers = self.build_headers()?;
        let mut request = self.build_request(message);
        request.stream = false;

        let response = self.client
            .post(&self.api_url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        tracing::debug!("Received response with status: {}", status);

        if !status.is_success() {
            return self.handle_error_response(response, status).await;
        }

        let response_data: GptResponse = response.json().await
            .map_err(|e| {
                tracing::error!("Failed to parse API response: {}", e);
                GptError::ParseError(e.to_string())
            })?;

        tracing::info!("Successfully received and parsed API response");
        response_data.choices.first()
            .and_then(|choice| choice.message.as_ref().map(|msg| msg.content.clone()))
            .ok_or_else(|| {
                tracing::error!("No response content available in API response");
                GptError::ParseError("No response content available".to_string())
            })
    }

    // Method for streaming requests
    pub async fn ask_stream(&self, message: &str) -> Result<ReceiverStream<Result<String, GptError>>, GptError> {
        tracing::info!("Starting streaming request to GPT API");

        let mut headers = self.build_headers()?;
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("text/event-stream"),
        );

        let mut request = self.build_request(message);
        request.stream = true;

        let response = self.client
            .post(&self.api_url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return self.handle_error_response(response, status).await;
        }

        let (tx, rx) = mpsc::channel(100);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                            buffer.push_str(&text);

                            while let Some(pos) = buffer.find("\n\n") {
                                let message = buffer[..pos].to_string();
                                buffer = buffer[pos + 2..].to_string();

                                if message.starts_with("data: ") {
                                    let data = message.trim_start_matches("data: ");
                                    if data == "[DONE]" {
                                        break;
                                    }

                                    match serde_json::from_str::<GptResponse>(data) {
                                        Ok(response) => {
                                            if let Some(choice) = response.choices.first() {
                                                if let Some(delta) = &choice.delta {
                                                    if let Some(content) = &delta.content {
                                                        if !content.is_empty() {
                                                            let _ = tx.send(Ok(content.clone())).await;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to parse stream data: {}", e);
                                            let _ = tx.send(Err(GptError::ParseError(e.to_string()))).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(GptError::RequestError(e))).await;
                        break;
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    // Helper method to handle error responses
    async fn handle_error_response<T>(&self, response: reqwest::Response, status: reqwest::StatusCode) -> Result<T, GptError> {
        let error_message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("API request failed: {} - {}", status, error_message);
        Err(GptError::ApiError {
            status_code: status.as_u16(),
            message: error_message,
        })
    }
}

// Builder implementation for creating client instances
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