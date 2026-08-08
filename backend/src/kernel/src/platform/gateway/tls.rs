use std::path::Path;
use std::sync::Arc;

use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};
use quinn::ServerConfig as QuinnServerConfig;
use quinn::crypto::rustls::QuicServerConfig;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::TlsAcceptor;

use super::BrowserGatewayConfigurationV1;

const TLS_MATERIAL_MAX_BYTES: u64 = 64 * 1024;

pub(super) fn acceptor(
    configuration: &BrowserGatewayConfigurationV1,
    alpn: Option<&[u8]>,
) -> Result<TlsAcceptor, String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certificate_path = configuration
        .certificate_der_path
        .as_ref()
        .ok_or_else(|| "browser Gateway TLS certificate is unavailable".to_owned())?;
    let private_key_path = configuration
        .private_key_der_path
        .as_ref()
        .ok_or_else(|| "browser Gateway TLS private key is unavailable".to_owned())?;
    let certificate = CertificateDer::from(read_material(certificate_path)?);
    let private_key = PrivateKeyDer::try_from(read_material(private_key_path)?)
        .map_err(|_| "browser Gateway private key is invalid".to_owned())?;
    let mut server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key)
        .map_err(|error| {
            format!("browser Gateway TLS certificate or private key is invalid: {error}")
        })?;
    if let Some(alpn) = alpn {
        server.alpn_protocols = vec![alpn.to_vec()];
    }
    Ok(TlsAcceptor::from(Arc::new(server)))
}

pub(super) fn http3_server_config(
    configuration: &BrowserGatewayConfigurationV1,
) -> Result<QuinnServerConfig, String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certificate_path = configuration
        .certificate_der_path
        .as_ref()
        .ok_or_else(|| "browser Gateway TLS certificate is unavailable".to_owned())?;
    let private_key_path = configuration
        .private_key_der_path
        .as_ref()
        .ok_or_else(|| "browser Gateway TLS private key is unavailable".to_owned())?;
    let certificate = CertificateDer::from(read_material(certificate_path)?);
    let private_key = PrivateKeyDer::try_from(read_material(private_key_path)?)
        .map_err(|_| "browser Gateway private key is invalid".to_owned())?;
    let mut crypto = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key)
        .map_err(|error| {
            format!("browser Gateway TLS certificate or private key is invalid: {error}")
        })?;
    crypto.alpn_protocols = vec![b"h3".to_vec()];
    crypto.max_early_data_size = 0;
    Ok(QuinnServerConfig::with_crypto(Arc::new(
        QuicServerConfig::try_from(crypto)
            .map_err(|_| "browser Gateway HTTP/3 TLS configuration is invalid".to_owned())?,
    )))
}

fn read_material(path: &Path) -> Result<Vec<u8>, String> {
    read_secure_file(
        path,
        SecureReadPolicy::owner_private(TLS_MATERIAL_MAX_BYTES),
    )
    .map_err(|_| "browser Gateway TLS material is unavailable".to_owned())
}
