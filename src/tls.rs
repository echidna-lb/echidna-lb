use crate::config::SslConfig;
use crate::error::EchidnaError;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

pub fn load_tls_config(ssl_config: &SslConfig) -> Result<ServerConfig, EchidnaError> {
    let cert_file = &mut BufReader::new(File::open(&ssl_config.cert_path)?);
    let key_file = &mut BufReader::new(File::open(&ssl_config.key_path)?);

    let certs = certs(cert_file)?;
    let keys = pkcs8_private_keys(key_file)?;

    let new_cert: Vec<Certificate> = certs.into_iter().map(Certificate).collect();
    let new_keys: Vec<PrivateKey> = keys.into_iter().map(PrivateKey).collect();

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(new_cert, new_keys[0].clone())
        .map_err(EchidnaError::from)
}
