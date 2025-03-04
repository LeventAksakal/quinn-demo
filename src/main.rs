use clap::Parser;
use std::error::Error;
mod cli;
mod client;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let cli_args = cli::Cli::parse();

    // Execute the appropriate command
    cli::run(cli_args).await
}
