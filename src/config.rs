#[derive(Debug, Clone)]
pub struct GptConfig {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub stop: Option<Vec<String>>,
}

impl Default for GptConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 800,
            top_p: 0.95,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop: None,
        }
    }
}

impl GptConfig {
    pub fn builder() -> GptConfigBuilder {
        GptConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct GptConfigBuilder {
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
    stop: Option<Vec<String>>,
}

impl GptConfigBuilder {
    pub fn temperature(mut self, temp: f32) -> Self {
        tracing::debug!("Setting temperature to: {}", temp);
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    pub fn max_tokens(mut self, tokens: u32) -> Self {
        tracing::debug!("Setting max_tokens to: {}", tokens);
        self.max_tokens = Some(tokens);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        tracing::debug!("Setting top_p to: {}", top_p);
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    pub fn frequency_penalty(mut self, penalty: f32) -> Self {
        tracing::debug!("Setting frequency_penalty to: {}", penalty);
        self.frequency_penalty = Some(penalty.clamp(-2.0, 2.0));
        self
    }

    pub fn presence_penalty(mut self, penalty: f32) -> Self {
        tracing::debug!("Setting presence_penalty to: {}", penalty);
        self.presence_penalty = Some(penalty.clamp(-2.0, 2.0));
        self
    }

    pub fn stop(mut self, stop: Vec<String>) -> Self {
        tracing::debug!("Setting stop sequences: {:?}", stop);
        self.stop = Some(stop);
        self
    }

    pub fn build(self) -> GptConfig {
        let default = GptConfig::default();
        tracing::info!("Building GPT configuration");
        GptConfig {
            temperature: self.temperature.unwrap_or(default.temperature),
            max_tokens: self.max_tokens.unwrap_or(default.max_tokens),
            top_p: self.top_p.unwrap_or(default.top_p),
            frequency_penalty: self.frequency_penalty.unwrap_or(default.frequency_penalty),
            presence_penalty: self.presence_penalty.unwrap_or(default.presence_penalty),
            stop: self.stop,
        }
    }
}
