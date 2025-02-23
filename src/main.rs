use anyhow::Result;
use std::env;
use std::io::{self, Write};
use explore::GptClient;

async fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize the GPT client
    let client = GptClient::new(
        env::var("AZUREOPENAI_API_URL")?,
        env::var("AZUREOPENAI_API_KEY")?,
    )?;

    println!("Welcome to GPT Client!");
    println!("Type 'exit' to quit the program.\n");

    loop {
        let input = get_user_input("You: ").await?;

        if input.to_lowercase() == "exit" {
            println!("Goodbye!");
            break;
        }

        match client.ask(&input).await {
            Ok(response) => println!("\nGPT: {}\n", response),
            Err(e) => eprintln!("Error: {}\n", e),
        }
    }

    Ok(())
}