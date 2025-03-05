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
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
pub const SERVER_NAME: &str = "localhost";
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);
#[allow(unused)] //Make use of Client attributes

/// Client for QUIC connection
///
/// This struct provides methods for establishing and managing QUIC connections
/// to a server using the Quinn library.
///
/// ### Fields
///
/// * `client_addr` - Socket address for the client
/// * `server_addr` - Socket address for the server
/// * `server_name` - Server name for TLS verification
///
/// ### Example
///
/// ```
/// let client = Client::new();
/// let endpoint = client.setup_client_endpoint()?;
/// let connection = client.connect_to_server(&endpoint).await?;
/// Client::handle_client_session(connection).await?;
/// ```
pub struct Client {
    client_addr: SocketAddr,
    server_addr: SocketAddr,
    server_name: String,
}
#[allow(unused)] // Make use of Client::new & Client::with_args

impl Client {
    pub fn new() -> Self {
        Client {
            client_addr: CLIENT_ADDR,
            server_addr: SERVER_ADDR,
            server_name: SERVER_NAME.to_string(),
        }
    }

    pub fn with_args(client_addr: SocketAddr, server_addr: SocketAddr, server_name: &str) -> Self {
        Client {
            client_addr,
            server_addr,
            server_name: server_name.to_string(), // Convert to String
        }
    }

    /// Handles a client session with a QUIC server
    ///
    /// This function manages the communication flow for a client session:
    /// 1. Opens a bidirectional stream
    /// 2. Sends a hello message to the server
    /// 3. Reads the server's response
    /// 4. Closes the connection
    ///
    /// ### Arguments
    ///
    /// * `connection` - The established QUIC connection with the server
    ///
    /// ### Returns
    ///
    /// A `Result` containing:
    /// * `Ok(())` - The session completed successfully
    /// * `Err(Box<dyn Error>)` - An error if any part of the session management fails
    pub async fn handle_client_session(connection: Connection) -> Result<(), Box<dyn Error>> {
        let (mut send, mut recv) = connection.open_bi().await?;
        let message = b"Hello, QUIC server!";
        send.write_all(message).await?;
        send.finish()?;

        println!("Sent message: {}", String::from_utf8_lossy(message));
        let mut buffer = vec![0; 1024];
        let bytes_read = recv.read(&mut buffer).await?.unwrap();

        println!(
            "Received response: {}",
            String::from_utf8_lossy(&buffer[..bytes_read])
        );

        connection.close(0u32.into(), b"Done");
        println!("Connection closed");

        Ok(())
    }
    ///  Setups a client endpoint for a quinn connection
    ///
    /// ### Arguments
    ///
    /// -
    ///
    /// ### Returns
    ///
    /// A `Result` containing:
    /// * `Ok(Endpoint)` - A successful quinn endpoint
    /// * `Err(Box<dyn Error>)` - An error if the anything fails
    pub fn setup_client_endpoint() -> Result<Endpoint, Box<dyn Error>> {
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

        let mut endpoint = Endpoint::client(CLIENT_ADDR)?;
        endpoint.set_default_client_config(client_config);
        Ok(endpoint)
    }

    /// Establishes a connection to a QUIC server
    ///
    /// ### Arguments
    ///
    /// * `endpoint` - A quinn endpoint from which the connection will be established
    ///
    /// ### Returns
    ///
    /// A `Result` containing:
    /// * `Ok(Connection)` - A successful quinn connection to the server
    /// * `Err(Box<dyn Error>)` - An error if the connection fails
    ///
    /// ### Example
    ///
    /// ```
    /// let endpoint = Client::setup_client_endpoint()?;
    /// let connection = Client::connect_to_server(&endpoint).await?;
    /// ```
    pub async fn connect_to_server(endpoint: &Endpoint) -> Result<Connection, Box<dyn Error>> {
        println!("Connecting to server at {}", SERVER_ADDR);

        let connecting = endpoint.connect(SERVER_ADDR, SERVER_NAME)?;
        let connection = connecting.await?;

        println!("Connected to server: {}", connection.remote_address());
        Ok(connection)
    }
}
