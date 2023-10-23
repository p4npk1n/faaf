use std::fmt;

use crate::database;


#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseTomlError(toml::de::Error),
    NoAccessSatement(),
    DatabaseError(database::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IO error: {}", err),
            Error::ParseTomlError(err) => write!(f, "Parse Toml File Error: {}", err),
            Error::NoAccessSatement() => write!(f, "there is access struct and is not access statement"),
            Error::DatabaseError(err) => write!(f, "database error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(err) => Some(err),
            Error::ParseTomlError(err) => Some(err),
            Error::NoAccessSatement() => None,
            Error::DatabaseError(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::ParseTomlError(err)
    }
}

impl From<database::error::Error> for Error{
    fn from(err: database::error::Error) -> Self {
        Error::DatabaseError(err)
    }
}