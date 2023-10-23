extern crate nom;
use serde_json;
use crate::config::parser::parser_type;

#[derive(Debug, PartialEq)]
pub struct Condition {
    pub left: parser_type::Value,
    pub op: parser_type::Operator,
    pub right: parser_type::Value,
    pub chain: Option<parser_type::Chain>,
}

impl From<&parser_type::LiteralValue> for serde_json::Value {
    fn from(value: &parser_type::LiteralValue) -> serde_json::Value {
        match value {
            parser_type::LiteralValue::Int(i) => serde_json::Value::Number((*i).into()),
            parser_type::LiteralValue::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0))),
            parser_type::LiteralValue::String(s) => serde_json::Value::String(s.clone()),
            parser_type::LiteralValue::Bool(b) => serde_json::Value::Bool(*b),
            parser_type::LiteralValue::Null => serde_json::Value::Null,
        }
    }
}

impl Into<serde_json::Value> for parser_type::LiteralValue {
    fn into(self) -> serde_json::Value {
        match self {
            parser_type::LiteralValue::Int(i) => serde_json::Value::Number(i.into()),
            parser_type::LiteralValue::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0))),
            parser_type::LiteralValue::String(s) => serde_json::Value::String(s),
            parser_type::LiteralValue::Bool(b) => serde_json::Value::Bool(b),
            parser_type::LiteralValue::Null => serde_json::Value::Null,
        }
    }
}


