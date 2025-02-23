// models.rs

use serde::{Deserialize, Serialize};

// Request-related models
/// Represents a message in the conversation with the GPT model.
/// Each message has a role (like "user" or "assistant") and content.
#[derive(Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// The main request structure sent to the GPT API.
/// This includes all parameters that control the model's behavior.
#[derive(Debug, Serialize)]
pub struct GptRequest {
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

// Response-related models
/// The main response structure received from the GPT API.
/// This includes an optional ID and a vector of choices.
#[derive(Debug, Deserialize)]
pub struct GptResponse {
    pub id: Option<String>,
    pub choices: Vec<Choice>,
}

/// A choice in the GPT response, which can contain either a complete message
/// or a delta update in streaming mode.
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Option<ResponseMessage>,
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
    pub index: i32,
}

/// A complete message in a non-streaming response.
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub content: String,
    pub role: Option<String>,
}

/// A partial update received during streaming.
#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
}