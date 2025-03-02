// - platform verifier delegates the tls verification to OS. It requires a cryptoProvider
// rustls::crypto::CryptoProvider::install_default(default_provider())
//     .expect("Failed to install crypto provider");
// let client_config = ClientConfig::with_platform_verifier();
// Load and parse the server's certificate
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use quinn::{ClientConfig, Connection, Endpoint};
use rustls::RootCertStore;
const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
const SERVER_NAME: &str = "localhost";
const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting QUIC client...");

    // Set up client endpoint with certificate configuration
    let endpoint = setup_client_endpoint()?;

    // Connect and establish a session
    let connection = connect_to_server(&endpoint).await?;

    // Handle the client session
    if let Err(e) = handle_client_session(connection).await {
        eprintln!("Session error: {}", e);
        return Err(e);
    }

    Ok(())
}

async fn handle_client_session(connection: Connection) -> Result<(), Box<dyn Error>> {
    // Open a bidirectional stream
    let (mut send, mut recv) = connection.open_bi().await?;
    // Send a test message
    let message = b"Hello, QUIC server!";
    send.write_all(message).await?;
    send.finish()?;
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

fn setup_client_endpoint() -> Result<Endpoint, Box<dyn Error>> {
    let mut cert_chain_reader = BufReader::new(File::open("cert.pem")?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)
        .into_iter()
        .filter_map(|cert| cert.ok())
        .map(|cert| rustls::pki_types::CertificateDer::from(cert))
        .collect::<Vec<_>>();
    let mut root_store = RootCertStore::empty();
    for cert in certs {
        root_store.add(cert)?;
    }
    let client_config =
        ClientConfig::with_root_certificates(std::sync::Arc::new(root_store)).unwrap();

    // Bind this endpoint to a UDP socket on the given client address.
    let mut endpoint = Endpoint::client(CLIENT_ADDR)?;

    // Set the client configuration
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

async fn connect_to_server(endpoint: &Endpoint) -> Result<Connection, Box<dyn Error>> {
    println!("Connecting to server at {}", SERVER_ADDR);

    // Connect to the server
    let connecting = endpoint.connect(SERVER_ADDR, SERVER_NAME)?;
    let connection = connecting.await?;

    println!("Connected to server: {}", connection.remote_address());
    Ok(connection)
}
