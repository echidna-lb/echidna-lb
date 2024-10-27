use serde_yaml::Error as SerdeError;
use std::fmt::{Debug, Display, Formatter};
use std::io::{self, Error as IoError};
use tokio_rustls::rustls::Error as RustlsError;

#[derive(Debug)]
pub enum EchidnaError {
    Io(IoError),
    Serde(SerdeError),
    Rustls(RustlsError),
    Custom(String),
}

impl Display for EchidnaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EchidnaError::Io(err) => {
                write!(f, "IO error: {err}")
            }
            EchidnaError::Serde(err) => {
                write!(f, "Serde yaml error: {err}")
            }
            EchidnaError::Rustls(err) => {
                write!(f, "Rustls error: {err}")
            }
            EchidnaError::Custom(err) => {
                write!(f, "{err}")
            }
        }
    }
}

impl std::error::Error for EchidnaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EchidnaError::Io(err) => err.source(),
            EchidnaError::Serde(err) => err.source(),
            EchidnaError::Rustls(err) => err.source(),
            EchidnaError::Custom(_err) => None,
        }
    }
}

impl From<IoError> for EchidnaError {
    fn from(err: IoError) -> EchidnaError {
        EchidnaError::Io(err)
    }
}

impl From<SerdeError> for EchidnaError {
    fn from(err: SerdeError) -> EchidnaError {
        EchidnaError::Serde(err)
    }
}

impl From<RustlsError> for EchidnaError {
    fn from(err: RustlsError) -> EchidnaError {
        EchidnaError::Rustls(err)
    }
}

impl From<&str> for EchidnaError {
    fn from(err: &str) -> EchidnaError {
        EchidnaError::Custom(String::from(err))
    }
}

impl From<EchidnaError> for io::Error {
    fn from(err: EchidnaError) -> io::Error {
        match err {
            EchidnaError::Io(e) => e,
            EchidnaError::Serde(e) => io::Error::new(io::ErrorKind::InvalidInput, e),
            EchidnaError::Rustls(e) => io::Error::new(io::ErrorKind::NotConnected, e),
            EchidnaError::Custom(e) => io::Error::new(io::ErrorKind::Other, e),
        }
    }
}