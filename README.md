# GPT Client Documentation for Rust Beginners

This document explains the implementation of a GPT client in Rust, designed to interact with Azure's OpenAI service. We'll explore how each component works together, including our new streaming capabilities and enhanced error handling.

## Project Structure

The project is organized into several files:
- `main.rs`: The entry point of the application, handling user interaction and streaming control
- `lib.rs`: Core client implementation with both streaming and non-streaming capabilities
- `error.rs`: Comprehensive error handling
- `config.rs`: Configuration management
- `models.rs`: Data structures for requests and responses, supporting both streaming and non-streaming formats

## Enhanced Features

### 1. Streaming Support

Our client now supports real-time streaming of responses, allowing you to see the text as it's generated. This creates a more interactive experience, especially useful for longer responses. Here's how it works:

```rust
// Enable streaming for immediate response display
pub async fn ask_stream(&self, message: &str) -> Result<ReceiverStream<Result<String, GptError>>, GptError> {
    let mut headers = self.build_headers()?;
    headers.insert(
        "Accept",
        header::HeaderValue::from_static("text/event-stream"),
    );
    
    // Rest of the streaming implementation...
}
```

The streaming feature provides several benefits:
- Real-time response display
- Lower latency for first word appearance
- Better user experience for long responses
- Immediate feedback during generation

To use streaming in your application:

```rust
// Toggle streaming mode on/off
if streaming_mode {
    let stream = client.ask_stream("Tell me a story").await?;
    while let Some(chunk) = stream.next().await {
        print!("{}", chunk?);
        io::stdout().flush()?;
    }
} else {
    let response = client.ask("Tell me a story").await?;
    println!("{}", response);
}
```

### 2. Enhanced Error Handling

We've implemented a more robust error handling system that provides detailed information about what went wrong and where. Our error types now include:

```rust
pub enum GptError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
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
```

This structured approach to error handling helps you:
- Identify the exact cause of failures
- Handle different error types appropriately
- Provide meaningful feedback to users
- Debug issues more effectively

### 3. Improved Response Models

Our response models now handle both streaming and non-streaming cases elegantly:

```rust
#[derive(Debug, Deserialize)]
pub struct GptResponse {
    pub id: Option<String>,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Option<ResponseMessage>,
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
    pub index: i32,
}
```

This structure allows us to:
- Handle both response types with a single model
- Maintain type safety throughout the application
- Properly deserialize streaming updates
- Keep the code clean and maintainable

## Using the Enhanced Client

1. Create a configuration with desired parameters:
```rust
let config = GptConfig::builder()
    .temperature(0.8)
    .max_tokens(1000)
    .build();
```

2. Initialize the client:
```rust
let client = GptClient::builder()
    .api_url(env::var("AZUREOPENAI_API_URL")?)
    .api_key(env::var("AZUREOPENAI_API_KEY")?)
    .config(config)
    .build()?;
```

3. Choose between streaming and non-streaming responses:
```rust
// For streaming responses
if let Ok(mut stream) = client.ask_stream("Your question here").await {
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(content) => print!("{}", content),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

// For complete responses
if let Ok(response) = client.ask("Your question here").await {
    println!("{}", response);
}
```

## Environment Setup

Your `.env` file should include:
```env
AZUREOPENAI_API_URL=your_api_url_here
AZUREOPENAI_API_KEY=your_api_key_here
RUST_LOG=debug  # Control logging level
```

Dependencies in `Cargo.toml` now include streaming support:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
futures = "0.3"
tokio-stream = "0.1"
# ... other dependencies remain the same
```

## Best Practices for Streaming

When working with streaming responses:

1. **Buffer Management**
   - Handle partial messages appropriately
   - Maintain state between chunks
   - Flush output regularly for smooth display

2. **Error Handling**
   - Handle stream interruptions gracefully
   - Provide feedback during streaming
   - Clean up resources properly

3. **User Experience**
   - Show progress indicators when appropriate
   - Handle user interruptions
   - Maintain consistent output formatting

## Advanced Usage

### Custom Stream Processing

You can implement custom stream processors for special handling:

```rust
pub async fn process_stream<F>(stream: ResponseStream, mut processor: F) -> Result<(), GptError>
where
    F: FnMut(&str) -> Result<(), GptError>,
{
    while let Some(chunk) = stream.next().await {
        processor(&chunk?)?;
    }
    Ok(())
}
```

### Timeouts and Cancellation

Add timeouts to streaming requests:

```rust
use tokio::time::{timeout, Duration};

let stream_result = timeout(
    Duration::from_secs(30),
    client.ask_stream("Your question")
).await??;
```

## Common Pitfalls and Solutions

1. **Stream Processing**
   - Always handle partial messages correctly
   - Don't assume complete sentences in chunks
   - Remember to flush output regularly

2. **Resource Management**
   - Clean up streams properly
   - Handle disconnections gracefully
   - Monitor memory usage with long streams

3. **Error Handling**
   - Implement proper error recovery
   - Log errors appropriately
   - Provide meaningful user feedback

The enhanced client provides a more robust and feature-rich experience while maintaining the simplicity and safety guarantees that Rust is known for. Whether you're building a chat interface, a code generation tool, or any other AI-powered application, these features give you the flexibility and reliability you need.