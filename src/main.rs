use anyhow::Result;
use std::env;
use std::io::{self, Write};
use explore::{GptClient, config::GptConfig};
use futures::StreamExt;
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
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .init();

    dotenv::dotenv().ok();

    let config = GptConfig::builder()
        .temperature(0.8)
        .max_tokens(1000)
        .build();

    let client = GptClient::builder()
        .api_url(env::var("AZUREOPENAI_API_URL")?)
        .api_key(env::var("AZUREOPENAI_API_KEY")?)
        .config(config)
        .build()?;

    println!("Welcome to Enhanced GPT Client!");
    println!("Type 'exit' to quit the program.");
    println!("Use '/stream' to toggle streaming mode (currently: OFF)\n");

    let mut streaming_mode = false;

    loop {
        let input = get_user_input("You: ").await?;

        match input.to_lowercase().as_str() {
            "exit" => {
                tracing::info!("User requested to exit the application");
                println!("Goodbye!");
                break;
            }
            "/stream" => {
                streaming_mode = !streaming_mode;
                tracing::info!("Streaming mode toggled to: {}", streaming_mode);
                println!("Streaming mode: {}", if streaming_mode { "ON" } else { "OFF" });
            }
            _ => {
                if streaming_mode {
                    match client.ask_stream(&input).await {
                        Ok(mut stream) => {
                            print!("GPT: ");
                            io::stdout().flush()?;

                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(content) => {
                                        print!("{}", content);
                                        io::stdout().flush()?;
                                    }
                                    Err(e) => {
                                        eprintln!("\nError in stream: {}", e);
                                        break;
                                    }
                                }
                            }
                            println!("\n");
                        }
                        Err(e) => {
                            tracing::error!("Error in streaming request: {}", e);
                            eprintln!("Error: {}\n", e);
                        }
                    }
                } else {
                    match client.ask(&input).await {
                        Ok(response) => {
                            println!("\nGPT: {}\n", response);
                        }
                        Err(e) => {
                            tracing::error!("Error processing request: {}", e);
                            eprintln!("Error: {}\n", e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}