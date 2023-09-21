extern crate nom;

// This parser is not fully in control, allowing for various types of input. 
// For example, it accepts strings and boolean values within arrays, and it interprets cases like 111aaa as just 111, 
// even if there are variables or characters following the numbers."

#[derive(Debug, PartialEq, Clone)]
pub enum AccessPath {
    Key(String),
    Index(Value),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Literal(LiteralValue),
    Access(String, Vec<AccessPath>),
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


pub fn parse_literal_value(input: &str) -> nom::IResult<&str, LiteralValue> {
    nom::branch::alt((
        parse_float,
        parse_int,
        parse_string,
        parse_bool,
        parse_null,
    ))(input)
}


pub fn parse_int(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input,(sign, digits)) = nom::sequence::tuple((
        nom::combinator::opt(
            nom::branch::alt((
                nom::character::complete::char('-'),
                nom::character::complete::char('+')
            ))
        ),
        nom::character::complete::digit1,
    ))(input)?;

    let num_str = match sign {
        Some('-') => format!("-{}", digits),
        Some('+') => digits.to_string(),
        None => digits.to_string(),
        _ => unreachable!(),
    };


    let value: i32= num_str.parse::<i32>().unwrap();
    return Ok((input, LiteralValue::Int(value)));
}

pub fn parse_float(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, sign) = nom::combinator::opt(
        nom::branch::alt((
            nom::character::complete::char('-'),
            nom::character::complete::char('+')
        ))
    )(input)?;

    let (input, integer_part) = nom::character::complete::digit1(input)?;
    let (input, _) = nom::bytes::complete::tag(".")(input)?;
    let (input, fractional_part) = nom::character::complete::digit1(input)?;

    let mut num_str = String::new();
    if let Some(s) = sign {
        num_str.push(s);
    }
    num_str.push_str(integer_part);
    num_str.push('.');
    num_str.push_str(fractional_part);

    let value: f64 = num_str.parse::<f64>().unwrap();
    return Ok((input, LiteralValue::Float(value)));
}

pub fn parse_string(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, value) = nom::sequence::delimited(
        nom::bytes::complete::tag("\""),
        nom::bytes::complete::is_not("\""),
        nom::bytes::complete::tag("\"")
    )(input)?;
    return Ok((input, LiteralValue::String(value.to_string())));
}


pub fn parse_bool(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, value) = nom::branch::alt((nom::bytes::complete::tag("true"), nom::bytes::complete::tag("false")))(input)?;
    match value {
        "true" => Ok((input, LiteralValue::Bool(true))),
        "false" => Ok((input, LiteralValue::Bool(false))),
        _ => unreachable!(),
    }
}

pub fn parse_access_path(input: &str) -> nom::IResult<&str, AccessPath> {
    nom::branch::alt((
        nom::combinator::map(
            nom::sequence::preceded(nom::character::complete::char('.'), nom::bytes::complete::take_while1(|c: char| c.is_alphanumeric() || c == '_') ),
            |s: &str| AccessPath::Key(s.to_string())
        ),
        nom::combinator::map(
            nom::sequence::preceded(nom::character::complete::char('['), nom::sequence::terminated(parse_value, nom::character::complete::char(']'))),
            |v: Value| AccessPath::Index(v)
        )
    ))(input)
}

pub fn parse_access(input: &str) -> nom::IResult<&str, Value> {
    let (input, base) =  nom::bytes::complete::take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, paths) = nom::multi::many0(parse_access_path)(input)?;
    Ok((input, Value::Access(base.to_string(), paths)))
}


pub fn parse_value(input: &str) -> nom::IResult<&str, Value> {
    nom::branch::alt((
        nom::combinator::map(parse_literal_value, |v| Value::Literal(v)),
        parse_access,   
    ))(input)
}


pub fn parse_null(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, _) = nom::bytes::complete::tag("null")(input)?;
    Ok((input, LiteralValue::Null))
}

pub fn parse_operator(i: &str) -> nom::IResult<&str, Operator> {
    let (i, op) = nom::branch::alt((
        nom::bytes::complete::tag("=="),
        nom::bytes::complete::tag("!="),
        nom::bytes::complete::tag("<="),
        nom::bytes::complete::tag(">="),
        nom::bytes::complete::tag("<"),
        nom::bytes::complete::tag(">"),
        nom::bytes::complete::tag("in"),
    ))(i)?;

    let operator = match op {
        "==" => Operator::Equal,
        "!=" => Operator::NotEqual,
        "<=" => Operator::LessThanEqual,
        ">=" => Operator::GreaterThanEqual,
        "<" => Operator::LessThan,
        ">" => Operator::GreaterThan,
        "in" => Operator::In,
        _ => unreachable!(),
    };
    Ok((i, operator))
}


pub fn parse_chain(i: &str) -> nom::IResult<&str, Option<Chain>> {
    
    let (i, chain) = nom::combinator::opt(nom::branch::alt((
        nom::bytes::complete::tag("and"),
        nom::bytes::complete::tag("or"),
    )))(i)?;

    let (i, _) = nom::character::complete::multispace0(i)?;

    let chain_enum = chain.map(|s| {
        match s {
            "and" => Chain::And,
            "or" => Chain::Or,
            _ => unreachable!(),
        }
    });

    Ok((i, chain_enum))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        let result = parse_int("123");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::Int(123))));

        let result = parse_int("-123");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::Int(-123))));

        let result = parse_int("123abc");
        println!("{:?}", result);
        assert_eq!(result, Ok(("abc", LiteralValue::Int(123))));
    }

    #[test]
    fn test_parse_float() {
        let result = parse_float("123.456");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::Float(123.456))));

        let result = parse_float("-123.456abc");
        println!("{:?}", result);
        assert_eq!(result, Ok(("abc", LiteralValue::Float(-123.456))));
    }

    #[test]
    fn test_parse_string() {
        let result = parse_string("\"hello\"");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::String("hello".to_string()))));
        
        let result = parse_string("\"hello\"world");
        println!("{:?}", result);
        assert_eq!(result, Ok(("world", LiteralValue::String("hello".to_string()))));
    }

    #[test]
    fn test_parse_bool() {
        let result = parse_bool("true");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::Bool(true))));

        let result = parse_bool("false");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::Bool(false))));
    }

    #[test]
    fn test_parse_access_path() {
        let result = parse_access_path(".keyName");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", AccessPath::Key("keyName".to_string()))));

        let result = parse_access_path("[123]");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", AccessPath::Index(Value::Literal(LiteralValue::Int(123))))));
    }

    #[test]
    fn test_parse_access() {
        let result = parse_access("object.key");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", Value::Access("object".to_string(), vec![AccessPath::Key("key".to_string())]))));
    }

    #[test]
    fn test_parse_value() {
        let result = parse_value("123");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", Value::Literal(LiteralValue::Int(123)))));
    }

    #[test]
    fn test_parse_null() {
        let result = parse_null("null");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", LiteralValue::Null)));
    }

    #[test]
    fn test_parse_operator() {
        let result = parse_operator("==");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", Operator::Equal)));
    }

    #[test]
    fn test_parse_chain() {
        let result = parse_chain("and");
        println!("{:?}", result);
        assert_eq!(result, Ok(("", Some(Chain::And))));
    }


}