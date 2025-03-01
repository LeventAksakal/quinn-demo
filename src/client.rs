use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use quinn::{ClientConfig, Endpoint};

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
const SERVER_NAME: &str = "localhost";
const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);

fn main() -> Result<(), Box<dyn Error>> {
    // Start the client (asynchronously)
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async { client().await })
}

async fn client() -> Result<(), Box<dyn Error>> {
    // Create client configuration
    let client_config = ClientConfig::with_platform_verifier();

    // Bind this endpoint to a UDP socket on the given client address.
    let mut endpoint = Endpoint::client(CLIENT_ADDR)?;

    // Set the client configuration
    endpoint.set_default_client_config(client_config);

    println!("Connecting to server at {}", SERVER_ADDR);

    // Connect to the server
    let connection = match endpoint.connect(SERVER_ADDR, SERVER_NAME) {
        Ok(conn) => conn,
        Err(e) => {
            println!("Failed to connect: {}", e);
            return Err(e.into());
        }
    };
    let connection = match connection.await {
        Ok(conn) => conn,
        Err(e) => {
            println!("Failed to connect: {}", e);
            return Err(e.into());
        }
    };
    println!("Connected to server: {}", connection.remote_address());

    // Open a bidirectional stream
    let (mut send, mut recv) = connection.open_bi().await?;
    println!("Bidirectional stream established");

    // Send a test message
    let message = b"Hello, QUIC server!";
    send.write_all(message).await?;
    println!("Sent message: {}", String::from_utf8_lossy(message));

    // Receive the response
    let mut buffer = vec![0; 1024];
    let bytes_read = recv.read(&mut buffer).await?.unwrap();
    println!(
        "Received response: {}",
        String::from_utf8_lossy(&buffer[..bytes_read])
    );

    // Properly close the connection
    connection.close(0u32.into(), b"Done");
    println!("Connection closed");

    Ok(())
}
