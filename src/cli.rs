use clap::{Parser, Subcommand};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Parser)]
#[command(author, version, about = "Quinn QUIC Demo Application")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run in server mode
    Server {
        /// Address to bind to in format IP:PORT (e.g., 127.0.0.1:5001)
        #[arg(short, long, value_parser = parse_socket_addr)]
        addr: Option<SocketAddr>,
    },

    /// Run in client mode
    Client {
        /// Server address to connect to in format IP:PORT (e.g., 127.0.0.1:5001)
        #[arg(short, long, value_parser = parse_socket_addr)]
        addr: Option<SocketAddr>,

        /// Server name for TLS validation
        #[arg(short, long)]
        server_name: Option<String>,
    },

    /// Run both server and client (demo mode)
    Demo,
}

// Custom parser function for SocketAddr
fn parse_socket_addr(s: &str) -> Result<SocketAddr, String> {
    SocketAddr::from_str(s).map_err(|e| format!("Invalid socket address: {}", e))
}

pub async fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        // If no subcommand is provided, run in demo mode (default behavior)
        None => run_demo().await,

        Some(Command::Server { addr }) => {
            run_server(addr).await?;
            Ok(())
        }

        Some(Command::Client { addr, server_name }) => {
            run_client(addr, server_name).await?;
            Ok(())
        }

        Some(Command::Demo) => run_demo().await,
    }
}

// Helper function to run the demo mode (server + client)
async fn run_demo() -> Result<(), Box<dyn Error>> {
    println!("Starting QUIC demo with both server and client...");

    // Setup the server endpoint
    let endpoint = match crate::server::Server::setup_server_endpoint() {
        Ok(endpoint) => {
            println!("Server listening on {}", crate::server::SERVER_ADDR);
            endpoint
        }
        Err(e) => {
            eprintln!("Failed to setup server: {}", e);
            return Err(e);
        }
    };

    // Spawn the server in a separate task
    let _server_handle = tokio::spawn(async move {
        println!("Server task started");
        if let Err(e) = crate::server::Server::server_loop(endpoint).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server a moment to start up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Starting QUIC client...");

    // Set up client endpoint with certificate configuration
    let endpoint = crate::client::Client::setup_client_endpoint()?;

    // Connect and establish a session
    let connection = crate::client::Client::connect_to_server(&endpoint).await?;

    // Handle the client session
    if let Err(e) = crate::client::Client::handle_client_session(connection).await {
        eprintln!("Session error: {}", e);
        return Err(e);
    }

    Ok(())
}
async fn run_server(addr: Option<SocketAddr>) -> Result<(), Box<dyn Error>> {
    let server_addr = addr.unwrap_or(crate::server::SERVER_ADDR);

    println!("Starting server on {}...", server_addr);

    // Setup the server endpoint with custom address if provided
    let endpoint = if server_addr == crate::server::SERVER_ADDR {
        crate::server::Server::setup_server_endpoint()?
    } else {
        // If you want to support custom addresses, you'd need to modify
        // Server::setup_server_endpoint to accept an address parameter
        // This is a placeholder for that functionality
        return Err("Custom server address not yet supported".into());
    };

    // Run the server loop
    crate::server::Server::server_loop(endpoint).await?;

    Ok(())
}
async fn run_client(
    addr: Option<SocketAddr>,
    server_name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let server_addr = addr.unwrap_or(crate::client::SERVER_ADDR);
    let server_name = server_name.unwrap_or_else(|| crate::client::SERVER_NAME.to_string());

    println!("Connecting to {} ({})...", server_addr, server_name);

    // Set up client endpoint with certificate configuration
    let endpoint = crate::client::Client::setup_client_endpoint()?;

    // Connect and establish a session
    // You'd need to modify connect_to_server to accept custom server_addr and server_name
    let connection =
        if server_addr == crate::client::SERVER_ADDR && server_name == crate::client::SERVER_NAME {
            crate::client::Client::connect_to_server(&endpoint).await?
        } else {
            // This is a placeholder for the functionality
            return Err("Custom client connection parameters not yet supported".into());
        };

    // Handle the client session
    crate::client::Client::handle_client_session(connection).await?;
    Ok(())
}
