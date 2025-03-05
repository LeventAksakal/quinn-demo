use quinn::{Endpoint, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);
/// Client for QUIC connection
///
/// This struct provides methods for establishing and managing QUIC connections
/// to a server using the Quinn library.
///
/// ### Fields
///
/// * `server_addr` - Socket address for the server
///
/// ### Example
///
/// ```
/// let server = Server::new();
/// let endpoint = servere.setup_client_endpoint()?;
/// let connection = server.connect_to_server(&endpoint).await?;
/// Server::handle_client_session(connection).await?;
/// ```
pub struct Server {
    #[allow(unused)]
    server_addr: SocketAddr,
}
impl Server {
    ///  Setup a server endpoint with default certificates and default configurations
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
    pub fn setup_server_endpoint() -> Result<Endpoint, Box<dyn Error>> {
        let (cert_chain, priv_key) = match Self::load_certificates_from_pem() {
            Ok((cert, key)) => (cert, key),
            Err(e) => {
                eprintln!("Failed to load certificates: {}", e);
                return Err(e);
            }
        };

        let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;

        let transport_config = {
            let mut config = TransportConfig::default();
            config.max_concurrent_uni_streams(0_u8.into());
            config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
            config
        };

        *Arc::get_mut(&mut server_config.transport).unwrap() = transport_config;

        let endpoint = Endpoint::server(server_config, SERVER_ADDR)?;

        Ok(endpoint)
    }

    /// Helper function to read .pem certificates into Der format
    fn load_certificates_from_pem(
    ) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn Error>> {
        let cert_file = File::open("cert.pem")?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
        let cert_chain = certs
            .into_iter()
            .map(CertificateDer::from)
            .collect::<Vec<_>>();

        if cert_chain.is_empty() {
            return Err("No certificates found in cert.pem".into());
        }

        let key_file = File::open("key.pem")?;
        let mut key_reader = BufReader::new(key_file);
        let key = match rustls_pemfile::private_key(&mut key_reader)? {
            Some(key) => PrivateKeyDer::from(key),
            None => return Err("No private key found in key.pem".into()),
        };

        Ok((cert_chain, key))
    }

    /// Basic server loop.
    /// Iterates over incoming connections and spawns a connection handler for each one.
    ///
    /// ### Arguments
    ///
    /// `endpoint` : Quinn endpoint for the server
    ///
    /// ### Returns
    ///
    /// - Ok(()) or Err(e)
    pub async fn server_loop(endpoint: Endpoint) -> Result<(), Box<dyn Error>> {
        println!("Waiting for incoming connections...");

        // Start iterating over incoming connections
        while let Some(conn) = endpoint.accept().await {
            match conn.await {
                Ok(connection) => {
                    println!("Connection received from {}", connection.remote_address());

                    // Handle the connection in a new task
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(connection).await {
                            eprintln!("Connection handling error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                    // Continue to accept other connections even if one fails
                }
            }
        }

        Ok(())
    }

    /// Connection handler for quinn connections.
    /// Spawns a stream handler for each bi-directional stream coming from client.
    pub async fn handle_connection(conn: quinn::Connection) -> Result<(), Box<dyn Error>> {
        println!("Handling connection from {}", conn.remote_address());

        while let Ok(stream) = conn.accept_bi().await {
            println!("Accepted new bidirectional stream");
            tokio::spawn(async move {
                if let Err(e) = Self::handle_stream(stream).await {
                    eprintln!("Stream handling error: {}", e);
                }
            });
        }

        println!("Connection closed by client: {}", conn.remote_address());
        Ok(())
    }

    /// Basic stream handler. Relays anything received back to client.
    pub async fn handle_stream(
        (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
    ) -> Result<(), Box<dyn Error>> {
        // Echo any data received
        let mut buf = vec![0; 1024];
        while let Ok(Some(n)) = recv.read(&mut buf).await {
            if n == 0 {
                break;
            }

            println!(
                "Received {} bytes: {}",
                n,
                String::from_utf8_lossy(&buf[..n])
            );

            match send.write_all(&buf[..n]).await {
                Ok(_) => println!("Echoed {} bytes back to client", n),
                Err(e) => {
                    eprintln!("Write failed: {}", e);
                    return Err(e.into());
                }
            }
        }

        println!("Stream closed");
        Ok(())
    }
}
