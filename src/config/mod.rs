use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::EchidnaError;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[clap(short, long, default_value = "./config.yaml")]
    pub config: PathBuf,
}

#[derive(Deserialize)]
pub struct Config {
    pub port: Option<u16>,
    pub debug: Option<bool>,
    pub https_port: Option<u16>,
    pub algorithm: String,
    pub workers: Option<usize>,
    pub healthcheck: Option<HealthcheckConfig>,
    pub backends: Vec<BackendConfig>,
    pub ssl: Option<SslConfig>,
}

#[derive(Deserialize)]
pub struct HealthcheckConfig {
    pub interval_sec: u64,
    pub route: String,
}

#[derive(Deserialize)]
pub struct BackendConfig {
    pub url: String,
    pub weight: usize,
}

#[derive(Debug, Deserialize)]
pub struct SslConfig {
    pub cert_path: String,
    pub key_path: String,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, EchidnaError> {
    let config_str = fs::read_to_string(path)?;
    serde_yaml::from_str(&config_str).map_err(EchidnaError::from)
}
