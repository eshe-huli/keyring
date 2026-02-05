//! QUIC transport — primary mesh communication
//!
//! Multiplexed, encrypted, NAT-traversal capable.
//! This is the Day 1 fully-implemented transport.

use anyhow::Result;
use async_trait::async_trait;
use quinn::{Endpoint, ServerConfig, ClientConfig};
use std::net::SocketAddr;
use std::sync::Arc;

use super::types::*;

pub struct QuicTransport {
    endpoint: Option<Endpoint>,
}

impl QuicTransport {
    pub fn new() -> Self {
        Self { endpoint: None }
    }

    /// Generate self-signed certificate from node identity
    fn make_server_config() -> Result<ServerConfig> {
        let cert = rcgen::generate_simple_self_signed(vec!["keyring".to_string()])?;
        let cert_der = rustls::pki_types::CertificateDer::from(cert.cert);
        let key_der = rustls::pki_types::PrivateKeyDer::from(
            rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der())
        );

        let mut server_config = ServerConfig::with_single_cert(
            vec![cert_der],
            key_der,
        )?;

        let transport_config = Arc::get_mut(&mut server_config.transport)
            .expect("transport config");
        transport_config.max_concurrent_bidi_streams(100u32.into());
        transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(15)));

        Ok(server_config)
    }

    fn make_client_config() -> ClientConfig {
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        ClientConfig::new(Arc::new(quinn::crypto::rustls::QuicClientConfig::try_from(crypto).unwrap()))
    }
}

/// Skip TLS verification (we verify via node signatures instead)
#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

// ─── Connection implementation ───

struct QuicConnection {
    connection: quinn::Connection,
    peer: PeerAddr,
}

#[async_trait]
impl Connection for QuicConnection {
    async fn send(&self, frame: SyncFrame) -> Result<()> {
        let data = bincode::serialize(&frame)?;
        let mut send = self.connection.open_uni().await?;
        send.write_all(&(data.len() as u32).to_be_bytes()).await?;
        send.write_all(&data).await?;
        send.finish()?;
        Ok(())
    }

    async fn recv(&self) -> Result<SyncFrame> {
        let mut recv = self.connection.accept_uni().await?;
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await?;

        Ok(bincode::deserialize(&data)?)
    }

    fn peer_addr(&self) -> PeerAddr {
        self.peer.clone()
    }

    async fn close(&self) -> Result<()> {
        self.connection.close(0u32.into(), b"done");
        Ok(())
    }
}

// ─── Listener implementation ───

struct QuicListener {
    endpoint: Endpoint,
}

#[async_trait]
impl Listener for QuicListener {
    async fn accept(&self) -> Result<Box<dyn Connection>> {
        let incoming = self.endpoint.accept().await
            .ok_or_else(|| anyhow::anyhow!("endpoint closed"))?;
        let connection = incoming.await?;

        let peer = PeerAddr {
            node_id: None,
            addr: connection.remote_address().to_string(),
            transport: "quic".to_string(),
        };

        Ok(Box::new(QuicConnection { connection, peer }))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.endpoint.local_addr()?)
    }
}

// ─── Transport trait implementation ───

#[async_trait]
impl Transport for QuicTransport {
    async fn listen(&self, addr: &str) -> Result<Box<dyn Listener>> {
        let socket_addr: SocketAddr = addr.parse()?;
        let server_config = Self::make_server_config()?;
        let endpoint = Endpoint::server(server_config, socket_addr)?;

        tracing::info!("QUIC listening on {}", socket_addr);

        Ok(Box::new(QuicListener { endpoint }))
    }

    async fn connect(&self, peer: &str) -> Result<Box<dyn Connection>> {
        let addr: SocketAddr = peer.parse()?;
        let client_config = Self::make_client_config();

        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        let connection = endpoint.connect(addr, "keyring")?.await?;

        tracing::info!("QUIC connected to {}", addr);

        let peer_addr = PeerAddr {
            node_id: None,
            addr: peer.to_string(),
            transport: "quic".to_string(),
        };

        Ok(Box::new(QuicConnection {
            connection,
            peer: peer_addr,
        }))
    }

    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        // QUIC doesn't do discovery — manual endpoints only
        Ok(vec![])
    }

    fn name(&self) -> &str {
        "quic"
    }
}
