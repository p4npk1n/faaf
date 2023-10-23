use nom::Err;
use rusqlite;
use std::fmt;

use crate::config;

#[derive(Debug)]
pub enum Error {
    RusqliteError(rusqlite::Error),
    NoneTransaction(),
    NotAnalyzerTable(),
    AnalyzerNameDoesNotHaveAnArray(),
    JsonArrayDoesNotHaveOtherThanInt(),
    UnimplementedError(),
    PathListDoesNotHaveAcess(),
    NotAnalyzerNameInDataBase(String),
    ComparisonErrorTypeMismatch(),
    DiffCondAndCondStmt(),
    NoAccessSatement(),
    CanNotMutableObject(),
    NoAnalyzerName(),
    BindRequired(),
    MismatchedBindType(),
    BindAlreadyProvided(),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::RusqliteError(err) => write!(f, "Rusqlite error: {}", err),
            Error::NoneTransaction() => write!(f, "none transaction"),
            Error::NotAnalyzerTable() => write!(f, "not analyzer table"),
            Error::AnalyzerNameDoesNotHaveAnArray() => write!(f, "analyzer name does not have array"),
            Error::JsonArrayDoesNotHaveOtherThanInt() => write!(f, "json array dose not have other than int"),
            Error::UnimplementedError() => write!(f, "unimplemented error"),
            Error::PathListDoesNotHaveAcess() => write!(f, "path list dose not have acess."),
            Error::NotAnalyzerNameInDataBase(analyzer_name) => write!(f, "analyzer name: `{}` is not in database", analyzer_name),
            Error::ComparisonErrorTypeMismatch() => write!(f, "comparison error."),
            Error::DiffCondAndCondStmt() => write!(f, "difference conditions struct and condition stmt"),
            Error::NoAccessSatement() => write!(f, "there is access struct and is not access statement"),
            Error::CanNotMutableObject() => write!(f, "can not mutable object"),
            Error::NoAnalyzerName() => write!(f, "no analyzer name"),
            Error::BindRequired() => write!(f, "bind required"),
            Error::MismatchedBindType() => write!(f, "missmatch bind type"),
            Error::BindAlreadyProvided() => write!(f, "bind already provided")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::RusqliteError(err) => Some(err),
            Error::NoneTransaction() => None,
            Error::NotAnalyzerTable() => None,
            Error::AnalyzerNameDoesNotHaveAnArray() => None,
            // in the featuer, this error is deleted. this program allow object in json array: test1[test2], test1[test2[test3]]
            Error::JsonArrayDoesNotHaveOtherThanInt() => None,
            Error::UnimplementedError() => None,
            Error::PathListDoesNotHaveAcess() => None,
            Error::NotAnalyzerNameInDataBase(str) => None,
            Error::ComparisonErrorTypeMismatch() => None,
            Error::DiffCondAndCondStmt() => None,
            Error::NoAccessSatement() => None,
            Error::CanNotMutableObject() => None,
            Error::NoAnalyzerName() => None,
            Error::BindRequired() => None,
            Error::MismatchedBindType() => None,
            Error::BindAlreadyProvided() => None,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::RusqliteError(err)
    }
}
