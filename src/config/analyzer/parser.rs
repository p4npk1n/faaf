extern crate nom;

// This parser is not fully in control, allowing for various types of input. 
// For example, it accepts strings and boolean values within arrays, and it interprets cases like 111aaa as just 111, 
// even if there are variables or characters following the numbers."

#[derive(Debug, PartialEq, Clone)]
enum LiteralValue {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq, Clone)]
enum AccessPath {
    Key(String),
    Index(Value),
}

#[derive(Debug, PartialEq, Clone)]
enum Value {
    Literal(LiteralValue),
    Access(String, Vec<AccessPath>),
}

#[derive(Debug, PartialEq)]
enum Operator {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    In,
}

#[derive(Debug, PartialEq)]
enum Chain {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
struct Condition {
    left: Value,
    op: Operator,
    right: Value,
    chain: Option<Chain>,
}

fn parse_condition(i: &str) -> nom::IResult<&str, Condition> {
    
    let (i, (_, left, _, op, _, right, _, chain, _)) =  nom::sequence::tuple((
        nom::character::complete::multispace0,
        parse_value,
        nom::character::complete::multispace0,
        parse_operator,
        nom::character::complete::multispace0,
        parse_value,
        nom::character::complete::multispace0,
        parse_chain,
        nom::character::complete::multispace0,
    ))(i)?;
    Ok((i, Condition { left, op, right, chain }))
}

fn parse_chain(i: &str) -> nom::IResult<&str, Option<Chain>> {
    
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

fn parse_literal_value(input: &str) -> nom::IResult<&str, LiteralValue> {
    nom::branch::alt((
        parse_float,
        parse_int,
        parse_string,
        parse_bool,
        parse_null,
    ))(input)
}


fn parse_int(input: &str) -> nom::IResult<&str, LiteralValue> {
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

fn parse_float(input: &str) -> nom::IResult<&str, LiteralValue> {
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

fn parse_string(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, value) = nom::sequence::delimited(
        nom::bytes::complete::tag("\""),
        nom::bytes::complete::is_not("\""),
        nom::bytes::complete::tag("\"")
    )(input)?;
    return Ok((input, LiteralValue::String(value.to_string())));
}


fn parse_bool(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, value) = nom::branch::alt((nom::bytes::complete::tag("true"), nom::bytes::complete::tag("false")))(input)?;
    match value {
        "true" => Ok((input, LiteralValue::Bool(true))),
        "false" => Ok((input, LiteralValue::Bool(false))),
        _ => unreachable!(),
    }
}



fn parse_access_path(input: &str) -> nom::IResult<&str, AccessPath> {
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

fn parse_access(input: &str) -> nom::IResult<&str, Value> {
    let (input, base) =  nom::bytes::complete::take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, paths) = nom::multi::many0(parse_access_path)(input)?;
    Ok((input, Value::Access(base.to_string(), paths)))
}


fn parse_value(input: &str) -> nom::IResult<&str, Value> {
    nom::branch::alt((
        nom::combinator::map(parse_literal_value, |v| Value::Literal(v)),
        parse_access,
    ))(input)
}


fn parse_null(input: &str) -> nom::IResult<&str, LiteralValue> {
    let (input, _) = nom::bytes::complete::tag("null")(input)?;
    Ok((input, LiteralValue::Null))
}

fn parse_operator(i: &str) -> nom::IResult<&str, Operator> {
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


#[cfg(test)]
mod tests {
    use super::*;

    fn test_parse_impl(i: &str) {
        let result = parse_condition(i);
        let condition = match result {
            Ok((_, c)) => {
                println!("{:?}", c);
                match c.right {
                    Value::Literal(LiteralValue::String(s)) => println!("{}", s),
                    _ => println!("The value is not a string literal"),
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
                panic!("Failed to parse condition");
            }
        };
    }

    #[test]
    fn test_parse_condition() {
        test_parse_impl(r#"aaaa == 1 "#);
        test_parse_impl(r#"aaaa == true "#);
        test_parse_impl(r#"aaaa == false "#);
        test_parse_impl(r#"aaaa == "aaaaa" "#);
        test_parse_impl(r#"aaaa == aaa "#);
        test_parse_impl(r#"aaaa == a_aa "#);
        test_parse_impl(r#"aaaa == a1aa "#);
        test_parse_impl(r#"aaaa == -1 "#);
        test_parse_impl(r#"aaaa == +1 "#);
        test_parse_impl(r#"aaaa == aaa[333].aaaaa.aaaa[4444]"#);
        test_parse_impl(r#"aaaa == aaa[]"#);
        test_parse_impl(r#"aaaa == aaa[aaaa]"#);
        test_parse_impl(r#"aaaa == aaa.aaa.aaa.aaa"#);

        // Test with the actual data being sent
        test_parse_impl(r#"basic_info.mime == "application/x-pie-executable""#);
        test_parse_impl(r#"basic_info.size > -5000.00a or"#);
    }
}
