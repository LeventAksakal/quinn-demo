use quinn::{Endpoint, ServerConfig, TransportConfig};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);

fn main() -> Result<(), Box<dyn Error>> {
    // Configure the server
    let server_config = configure_server()?;

    // Start the server (asynchronously)
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async { server(server_config).await })
}

async fn server(config: ServerConfig) -> Result<(), Box<dyn Error>> {
    // Bind this endpoint to a UDP socket on the given server address.
    let endpoint = Endpoint::server(config, SERVER_ADDR)?;
    println!("Server listening on {}", SERVER_ADDR);

    // Start iterating over incoming connections.
    while let Some(conn) = endpoint.accept().await {
        let connection = conn.await?;
        println!("Connection received from {}", connection.remote_address());

        // Handle the connection in a new task
        tokio::spawn(async move { handle_connection(connection).await });
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Connection) {
    // Basic handler that accepts a bidirectional stream
    if let Ok((mut send, mut recv)) = conn.accept_bi().await {
        println!("Stream established with {}", conn.remote_address());

        // Echo any data received
        let mut buf = vec![0; 1024];
        while let Ok(Some(n)) = recv.read(&mut buf).await {
            if n == 0 {
                break;
            }

            println!("Received {} bytes", n);
            if let Err(e) = send.write_all(&buf[..n]).await {
                println!("Write failed: {}", e);
                break;
            }
        }

        println!("Stream closed");
    }
}

fn configure_server() -> Result<ServerConfig, Box<dyn Error>> {
    // Parse the certificate and key
    let certs: Vec<CertificateDer> =
        CertificateDer::pem_file_iter("cert.pem")?.collect::<Result<_, _>>()?;
    let key = PrivateKeyDer::from_pem_file("key.pem")?;

    // Create server config
    let mut server_config = ServerConfig::with_single_cert(certs, key)?;

    // Configure transport parameters
    let mut transport_config = TransportConfig::default();
    transport_config.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into()?));
    server_config.transport_config(Arc::new(transport_config));

    Ok(server_config)
}
