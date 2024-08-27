use crate::config::SslConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use std::fs::File;
use std::io::BufReader;
use crate::error::EchidnaError;

pub fn load_tls_config(ssl_config: &SslConfig) -> Result<ServerConfig, EchidnaError> {
  let cert_file = &mut BufReader::new(File::open(&ssl_config.cert_path).expect("Unable to open cert file"));
  let key_file = &mut BufReader::new(File::open(&ssl_config.key_path).expect("Unable to open key file"));

  let certs = certs(cert_file).unwrap();
  let keys = pkcs8_private_keys(key_file).unwrap();

  let new_cert: Vec<Certificate> = certs.into_iter().map(Certificate).collect();
  let new_keys: Vec<PrivateKey> = keys.into_iter().map(PrivateKey).collect();

  Ok(ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(new_cert, new_keys[0].clone())?)
}