extern crate nom;
use crate::config::parser::parser_type;

#[derive(Debug, PartialEq)]
pub struct Argument {
    pub value: parser_type::Access,
}
