extern crate nom;


#[derive(Debug, PartialEq, Clone)]
pub enum AccessPath {
    Key(String),
    Index(IndexValue),
}

#[derive(Debug, PartialEq, Clone)]
pub enum IndexValue{
    Access(Access),
    Int(i32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access{
    pub base: String, 
    pub path: Option<Vec<AccessPath>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Literal(LiteralValue),
    Access(Access),
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq)]
pub enum Chain {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    In,
}

