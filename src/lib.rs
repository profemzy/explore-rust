use reqwest::{Client, header};
use serde_json::{json, Value};
use anyhow::Result;

pub struct GptClient {
    client: Client,
    api_url: String,
    api_key: String,
}

impl GptClient {
    pub fn new(api_url: String, api_key: String) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_url,
            api_key,
        })
    }

    pub async fn ask(&self, message: &str) -> Result<String> {
        let mut headers = header::HeaderMap::new();
        headers.insert("api-key", header::HeaderValue::from_str(&self.api_key)?);
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let body = json!({
            "messages": [{
                "role": "user",
                "content": message
            }],
            "temperature": 0.7,
            "max_tokens": 800,
            "top_p": 0.95,
            "frequency_penalty": 0,
            "presence_penalty": 0,
            "stop": null
        });

        let response = self.client
            .post(&self.api_url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;
            Ok(response_json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            Err(anyhow::anyhow!(
                "API request failed with status: {}",
                response.status()
            ))
        }
    }
}