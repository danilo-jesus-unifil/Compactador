use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CoreError {
    InvalidInput(String),
    InvalidConfiguration(String),
    Unsupported(String),
    Cancelled,
    Io(io::Error),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "entrada inválida: {message}"),
            Self::InvalidConfiguration(message) => {
                write!(formatter, "configuração inválida: {message}")
            }
            Self::Unsupported(message) => write!(formatter, "operação não suportada: {message}"),
            Self::Cancelled => formatter.write_str("operação cancelada"),
            Self::Io(error) => write!(formatter, "erro de I/O: {error}"),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for CoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
