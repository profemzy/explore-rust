use anyhow::Result;
use std::env;
use std::io::{self, Write};
use explore::{GptClient, config::GptConfig};
use tracing_subscriber::EnvFilter;

async fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables first
    dotenv::dotenv().ok();

    // Set up logging with RUST_LOG from .env, defaulting to "info" if not set
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(rust_log)))
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .init();

    tracing::info!("Starting GPT Client application");
    tracing::debug!("Loaded environment variables");

    // Create a custom configuration
    let config = GptConfig::builder()
        .temperature(0.8)
        .max_tokens(1000)
        .build();

    // Initialize the GPT client with builder pattern
    let client = GptClient::builder()
        .api_url(env::var("AZUREOPENAI_API_URL")?)
        .api_key(env::var("AZUREOPENAI_API_KEY")?)
        .config(config)
        .build()?;

    tracing::info!("GPT Client initialized successfully");

    println!("Welcome to Enhanced GPT Client!");
    println!("Type 'exit' to quit the program.\n");

    loop {
        let input = get_user_input("You: ").await?;

        if input.to_lowercase() == "exit" {
            tracing::info!("User requested to exit the application");
            println!("Goodbye!");
            break;
        }

        tracing::debug!("Processing user input of length: {}", input.len());

        match client.ask(&input).await {
            Ok(response) => {
                tracing::debug!("Received response of length: {}", response.len());
                println!("\nGPT: {}\n", response);
            }
            Err(e) => {
                tracing::error!("Error processing request: {}", e);
                eprintln!("Error: {}\n", e);
            }
        }
    }

    tracing::info!("Application shutting down");
    Ok(())
}