use nom::Err;
use std::fmt;
use crate::config::error::Error as ConfigError;
use crate::database::error::Error as DatabaseError;

use super::dispatcher;

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
    DatabaseError(DatabaseError),
    IoError(std::io::Error),
    DiffCondAndCondStmt(),
    DispathcerError(dispatcher::error::Error)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigError(err) => write!(f, "config error {}", err),
            Error::DatabaseError(err) => write!(f, "Databse Error: {}", err),
            Error::IoError(err) => write!(f, "IO error: {}", err),
            Error::DiffCondAndCondStmt() => write!(f, "difference conditions struct and condition stmt"),
            Error::DispathcerError(err) => write!(f, "dispather error {} ", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ConfigError(err) => Some(err),
            Error::DatabaseError(err) => Some(err),
            Error::IoError(err) => Some(err),
            Error::DiffCondAndCondStmt() => None,
            Error::DispathcerError(err) => Some(err),
        }
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::ConfigError(err)
    }
}

impl From<DatabaseError> for Error {
    fn from(err: DatabaseError) -> Self {
        Error::DatabaseError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<dispatcher::error::Error> for Error {
    fn from(err: dispatcher::error::Error) -> Self {
        Error::DispathcerError(err)
    }
}