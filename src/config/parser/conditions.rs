extern crate nom;
use crate::config::parser::base_new;

#[derive(Debug, PartialEq)]
pub struct Condition {
    left: base_new::Value,
    op: base_new::Operator,
    right: base_new::Value,
    chain: Option<base_new::Chain>,
}

#[derive(Debug, PartialEq)]
pub enum ParseConditionError<'a>{
    BaseParseError(base_new::ParseError<'a>),
    SyntaxError(base_new::ParseInput<'a>, &'a str),
    InvalidValue(base_new::ParseInput<'a>, &'a str),
    InvalidOperator(base_new::ParseInput<'a>, &'a str),
    InvalidChain(base_new::ParseInput<'a>, &'a str)
}

impl<'a> From<base_new::ParseError<'a>> for ParseConditionError<'a> {
    fn from(err: base_new::ParseError<'a>) -> ParseConditionError<'a> {
        ParseConditionError::BaseParseError(err)
    }
}

impl<'a> std::fmt::Display for ParseConditionError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseConditionError::BaseParseError(err) => write!(f, "{}", err),
            ParseConditionError::SyntaxError(input, add_info) => write!(f, "Syntax Error: input: {},  error info :{:?}", input, add_info),
            ParseConditionError::InvalidValue(input, add_info) => write!(f, "Invaild Value: input: {},  error info :{:?}", input, add_info),
            ParseConditionError::InvalidOperator(input, add_info) => write!(f, "Invaild Operator: input: {},  error info :{:?}", input, add_info),
            ParseConditionError::InvalidChain(input, add_info) => write!(f, "Invaild Chain: input: {},  error info :{:?}", input, add_info)
        }
    }
}

impl<'a> std::error::Error for ParseConditionError<'a> {}


pub fn parse_condition(input: base_new::ParseInput) -> Result<Condition, ParseConditionError<'_>> {
    let start: base_new::ParseInput = input;

    // [value] [multispace] [operator] [multispace] [value] [multispace] [option<chain>] [\n or EOF]
    let (remaining, space) = base_new::parse_whitespace(input)?;
    let (remaining, left) = base_new::parse_value(remaining)?;
    if remaining.is_empty() && !left.is_none(){
        // 111 EOF
        return Err(ParseConditionError::SyntaxError(remaining, "Truncated expression: Expected operator and right-hand value after left-hand value"));
    }
    if !remaining.is_empty() && left.is_none(){
        // aaaa == "aaaa"
        return Err(ParseConditionError::InvalidValue(remaining, "Invalid left-hand value"));
    }
    if remaining.is_empty() && left.is_none(){
        return Err(ParseConditionError::SyntaxError(remaining, "Missing left-hand value"));
    }


    // [multispace] [operator] [multispace] [value] [multispace] [option<chain>] [\n or EOF]
    let (remaining, space) = base_new::parse_whitespace(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !space.is_none() && remaining.is_empty(){
        return Err(ParseConditionError::SyntaxError(remaining, "Truncated expression: Expected operator and right-hand value after left-hand value"));
    }
    if space.is_none() && !remaining.is_empty(){
        // for exsample if `111aaa` is parsed as int type, remainig = aaa, result = Value::Int(111).
        return Err(ParseConditionError::InvalidValue(remaining, "Unexpected data exists in left value suffix"));
    }


    // [operator] [multispace] [value] [multispace] [option<chain>] [\n or EOF]
    let (remaining, op) = base_new::parse_operator(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !op.is_none() && remaining.is_empty(){
        // 1111 == EOF
        return Err(ParseConditionError::SyntaxError(remaining, "Truncated expression: Expected operator and right-hand value after left-hand value"));
    }
    if op.is_none() && !remaining.is_empty(){
        // 1111 ffff 1111
        return Err(ParseConditionError::InvalidOperator(remaining, "Invalid Operator"));
    }

    
    // [multispace] [value] [multispace] [option<chain>] [\n or EOF]
    let (remaining, space) = base_new::parse_whitespace(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !space.is_none() && remaining.is_empty(){
        // 111 == space EOF
        return Err(ParseConditionError::SyntaxError(remaining, "Truncated expression: Expected operator and right-hand value after left-hand value"));
    }
    if space.is_none() && !remaining.is_empty(){
        // 111 ==4444 1111. between operator and value is required space.
        return Err(ParseConditionError::InvalidOperator(remaining, "Unexpected data exists in operator suffix"));
    }


    // [value] [multispace] [option<chain>] [\n or EOF]
    let (remaining, right) = base_new::parse_value(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !right.is_none() && remaining.is_empty(){
        // 111 == 111
        // success
        return Ok( Condition { 
            left:left.unwrap(), 
            op:op.unwrap(), 
            right:right.unwrap(), 
            chain:None }
        );
    }
    if right.is_none() && !remaining.is_empty(){
        // "aaa" == aaa
        return Err(ParseConditionError::InvalidValue(remaining, "Invalid right-hand value"));
    }


    //[multispace] [option<chain>] [\n or EOF]
    let (remaining, space) = base_new::parse_whitespace(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !space.is_none() && remaining.is_empty(){
        // 111 == 111 space
        // success
        return Ok( Condition { 
            left:left.unwrap(), 
            op:op.unwrap(), 
            right:right.unwrap(), 
            chain:None }
        );
    }
    if space.is_none() && !remaining.is_empty(){
        // 111 == 111aaa
        return Err(ParseConditionError::InvalidValue(remaining, "Unexpected data exists in right value suffix"));
    }


    // [option<chain>] [\n or EOF]
    let (remaining, chain) = base_new::parse_chain(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !chain.is_none() && remaining.is_empty(){
        // 111 == 111 and
        // success
        return Ok( Condition { 
            left:left.unwrap(), 
            op:op.unwrap(), 
            right:right.unwrap(), 
            chain:chain }
        );
    }
    if chain.is_none() && !remaining.is_empty(){
        // 111 == 111 tttt
        return Err(ParseConditionError::InvalidChain(remaining, "Invalid chain"));
    }


    //[\n or EOF]
    let (remaining, space) = base_new::parse_whitespace(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !space.is_none() && remaining.is_empty(){
        // 111 == 111 and space
        // success
        return Ok( Condition { 
            left:left.unwrap(), 
            op:op.unwrap(), 
            right:right.unwrap(), 
            chain:chain }
        );
    }
    if space.is_none() && !remaining.is_empty(){
        // 111 == 111 andaaaa
        return Err(ParseConditionError::InvalidChain(remaining, "Unexpected data exists in right Chain suffix"));
    }

    // 111 == 111 and faddsaf
    return Err(ParseConditionError::SyntaxError(remaining, "Data exists at the rear of Chain"));

}


#[cfg(test)]
mod tests {
    use super::*;
    use base_new::{Value, Operator, AccessPath};

    #[test]
    fn test_parse_condition_valid_case() {
        let input = "a == b";
        let expected = Ok(Condition {
            left: base_new::Value::Access(vec![base_new::AccessPath::Key("a".to_string())]),
            op: base_new::Operator::Equal,
            right: base_new::Value::Access(vec![base_new::AccessPath::Key("b".to_string())]),
            chain: None,
        });
        assert_eq!(parse_condition(input), expected);
    }

    #[test]
    fn test_parse_condition_missing_right_value() {
        let input = "a ==";
        let expected = Err(ParseConditionError::SyntaxError("", "Truncated expression: Expected operator and right-hand value after left-hand value"));
        assert_eq!(parse_condition(input), expected);
    }

    #[test]
    fn test_parse_condition_with_chain() {
        let input = "a == b and";
        let expected = Ok(Condition {
            left: base_new::Value::Access(vec![base_new::AccessPath::Key("a".to_string())]),
            op: base_new::Operator::Equal,
            right: base_new::Value::Access(vec![base_new::AccessPath::Key("b".to_string())]),
            chain: Some(base_new::Chain::And),
        });
        assert_eq!(parse_condition(input), expected);
    }

    #[test]
    fn test_parse_condition_missing_left_value() {
        let input = "== a";
        let expected = Err(ParseConditionError::InvalidValue("== a", "Invalid left-hand value"));
        assert_eq!(parse_condition(input), expected);
    }

    #[test]
    fn test_parse_condition_invalid_chain() {
        let input = "a == b andand";
        let expected = Err(ParseConditionError::InvalidChain("and", "Unexpected data exists in right Chain suffix"));
        assert_eq!(parse_condition(input), expected);
    }

    #[test]
    fn test_parse_condition_missing_operator() {
        let input = "a a";
        let expected = Err(ParseConditionError::InvalidOperator("a", "Invalid Operator"));
        assert_eq!(parse_condition(input), expected);
    }

    #[test]
    fn test_parse_condition_invalid_right_value() {
        let input = "a == ==";
        let expected = Err(ParseConditionError::InvalidValue("==", "Invalid right-hand value"));
        assert_eq!(parse_condition(input), expected);
    }



}

