use crate::database::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;