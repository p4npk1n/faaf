use std::fmt;


#[derive(Debug)]
pub enum Error {
    UndefinedExtensionError(),
    PythonError(pyo3::prelude::PyErr),
    JsonError(serde_json::Error),
    SoError(libloading::Error),
    ShError(std::io::Error),
    SoPanicError(),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UndefinedExtensionError() => write!(f, "undefined extension error"),
            Error::PythonError(err) => write!(f, "python error {}", err),
            Error::JsonError(err) => write!(f, "json error {}", err),
            Error::SoError(err) => write!(f, "so error {}", err),
            Error::SoPanicError() => write!(f, "so panic error"),
            Error::ShError(err) => write!(f, "sh error {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::UndefinedExtensionError() => None,
            Error::PythonError(err) => Some(err),
            Error::JsonError(err) => Some(err),
            Error::SoError(err) => Some(err),
            Error::SoPanicError() =>  None,
            Error::ShError(err) =>  Some(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ShError(err)
    }
}

impl From<pyo3::prelude::PyErr> for Error {
    fn from(err: pyo3::prelude::PyErr) -> Self {
        Error::PythonError(err)
    }
}

impl From<libloading::Error> for Error {
    fn from(err: libloading::Error) -> Self {
        Error::SoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err)
    }
}