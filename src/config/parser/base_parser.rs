use super::parser_type::{AccessPath, IndexValue, Access, Value, LiteralValue, Chain, Operator, };
extern crate nom;


// This parser is not fully in control, allowing for various types of input. 
// For example, it accepts strings and boolean values within arrays, and it interprets cases like 111aaa as just 111, 
// even if there are variables or characters following the numbers."

pub type ParseInput<'a> = &'a str;
pub type ParseResult<OutputType> = Option<OutputType>;

#[derive(Debug, PartialEq)]
pub enum ParseError<'a>{
    Failure(ParseErrorWrapper<'a>),
    Str2Digits(std::num::ParseIntError),
    AccessPathDotError(ParseInput<'a>),
    InvalidDataInArray(ParseInput<'a>),
    UnmatchedClosingBracket(ParseInput<'a>)
}

#[derive(Debug, PartialEq)]
pub struct ParseErrorWrapper<'a>{
    add_info: &'a str,
    wrapper: nom::error::Error<&'a str>,
}

impl<'a> std::fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::Failure(wrapper) => write!(f, "{} : {:?}", wrapper.add_info, wrapper.wrapper),
            ParseError::Str2Digits(err) => write!(f, "{:?}", err),
            ParseError::AccessPathDotError(err) => write!(f, "error occurred after parsing the `.`: {:?}", err),
            ParseError::InvalidDataInArray(err) => write!(f, "invalid data in array: {:?}", err),
            ParseError::UnmatchedClosingBracket(err) => write!(f, "unmatched closing bracket {:?}", err)
        }
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

pub fn parse_whitespace(input: ParseInput) -> Result<(ParseInput, ParseResult<&str>), ParseError<'_>>{
    let result: nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = nom::character::complete::multispace0(input);
    let err_handle1: bool = handle_fatal_parse_error(&result, "invalid parse whitespace")?;
    if err_handle1 == false {
        panic!("Unexpected error: whitespace");
    }

    let (tail, output): (&str, &str) = result.unwrap();
    if output.is_empty(){
        return Ok((tail, None));
    }

    return Ok((tail, Some(output)));
}

fn is_peek_whitespace(input: ParseInput) -> Result<bool, ParseError<'_>> {
    let whitespace_parser: fn(char) -> bool = |c: char| c.is_whitespace();
    let result: nom::IResult<ParseInput, char, nom::error::Error<ParseInput>> = nom::character::complete::satisfy(whitespace_parser)(input);
    let err_handle1: bool = handle_fatal_parse_error(&result, "invalid parse whitespace")?;
    if err_handle1 == false {
        return Ok( false );
    }
    return Ok(true);
}


pub fn parse_literal_value(input: &str) -> Result<(ParseInput, ParseResult<LiteralValue>), ParseError<'_>> {

    let (input, output): (ParseInput, ParseResult<f64>) = parse_float(input)?;
    if output.is_some() {
        return Ok(
            ( 
                input, 
                Some(LiteralValue::Float(output.unwrap()))
            )
        );
    }

    let (input, output): (ParseInput, ParseResult<i32>) = parse_int(input)?;
    if output.is_some() {
        return Ok(
            ( 
                input, 
                Some(LiteralValue::Int(output.unwrap()))
            )
        );
    }

    let (input, output): (ParseInput, ParseResult<String>) = parse_string(input)?;
    if output.is_some() {
        return Ok(
            ( 
                input, 
                Some(LiteralValue::String(output.unwrap()))
            )
        );
    }

    let (input, output): (ParseInput, ParseResult<bool>) = parse_bool(input)?;
    if output.is_some() {
        return Ok(
            ( 
                input, 
                Some(LiteralValue::Bool(output.unwrap()))
            )
        );
    }

    let (input, output): (ParseInput, ParseResult<LiteralValue>) = parse_null(input)?;
    if output.is_some() {
        return Ok(
            ( 
                input, 
                Some(output.unwrap())
            )
        );
    }

    return Ok((input, None));

}

pub fn parse_digit(input: ParseInput) -> Result<(ParseInput, ParseResult<i32>), ParseError<'_>> {
    let start: ParseInput = input;

    let digits_result: nom::IResult<&str, &str, nom::error::Error<&str>> =
        nom::character::complete::digit1(input);

    let err_handle: bool = handle_fatal_parse_error(&digits_result, "invalid digit")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, output) = digits_result.unwrap();
    let digit_int: i32 = output.parse().map_err(ParseError::Str2Digits)?;

    return Ok( (input, Some(digit_int) ));

}

pub fn parse_int(input: ParseInput) -> Result<(ParseInput, ParseResult<i32>), ParseError<'_> >{
    let start: ParseInput = input;

    let (input, sign) = parse_sign(input)?;

    let (input, digits) = parse_digit(input)?;
    if digits.is_none() {
        return Ok( (start, None) );
    }

    let output: i32 = match sign {
        Some('-') => -digits.unwrap(),
        Some('+') => digits.unwrap(),
        None => digits.unwrap(),
        _ => panic!("Unexpected sign character"),
    };

    return Ok( (input, Some( output ) ) );

}

pub fn parse_sign(input: ParseInput) -> Result< (ParseInput, ParseResult<char>), ParseError<'_> > {

    let minus_parser: fn(&str) -> nom::IResult<&str, char, nom::error::Error<&str>> = |i: &str| nom::character::complete::char('-')(i);
    let plus_parser: fn(&str) -> nom::IResult<&str, char, nom::error::Error<&str>> = |i: &str| nom::character::complete::char('+')(i);
    let sign_result: nom::IResult<&str, Option<char>, nom::error::Error<&str>> = 
        nom::combinator::opt(nom::branch::alt((minus_parser, plus_parser)))(input);
    
    handle_fatal_parse_error(&sign_result, "invalid sign")?;

    return Ok(sign_result.unwrap());
}

pub fn parse_dot(input: ParseInput) -> Result<(ParseInput, ParseResult<char>), ParseError<'_> > {
    let start: ParseInput = input;

    let dot_result: nom::IResult<&str, char, nom::error::Error<&str>> = nom::character::complete::char('.')(input);

    let err_handle: bool = handle_fatal_parse_error(&dot_result, "invalid dot")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, _) = dot_result.unwrap();
    return Ok( ( input, Some('.') ) );
}

pub fn parse_null(input: &str) -> Result<(ParseInput, ParseResult<LiteralValue>), ParseError<'_> > {
    let start: ParseInput = input;

    let null_result: nom::IResult<&str, &str, nom::error::Error<&str>> = nom::bytes::complete::tag("null")(input);
    let err_handle: bool = handle_fatal_parse_error(&null_result, "invalid null")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, _) = null_result.unwrap();

    return Ok( ( input, Some(LiteralValue::Null) ) );
}

pub fn handle_fatal_parse_error<'a, T>(
    parse_result: &nom::IResult<&'a str, T, nom::error::Error<&'a str>>, add_info: &'a str) -> Result<bool, ParseError<'a>> {
    if let Err(err) = parse_result {
        match err {
            nom::Err::Failure(e) => {
                //let tmp: nom::error::Error<&str> = nom::error::Error {input: e.input.clone(), code: e.code.clone()};
                let tmp: nom::error::Error<&str> = nom::error::Error {input: e.input, code: e.code.clone()};
                return Err(ParseError::Failure(ParseErrorWrapper { add_info: add_info, wrapper: tmp}));
            }
            // Return false except for errors that prevent parsing of the next string.
            _ => return Ok(false),
        }
    }
    Ok(true)
}

pub fn parse_bool(input: ParseInput) -> Result<(ParseInput, ParseResult<bool>), ParseError<'_> > {
    let start: ParseInput = input;

    let true_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput|nom::bytes::complete::tag("true")(i);
    let false_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput|nom::bytes::complete::tag("false")(i);
    let bool_result: nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = nom::branch::alt((true_parser, false_parser))(input);

    let err_handle: bool = handle_fatal_parse_error(&bool_result, "invalid bool")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, output_str): (&str, &str) = bool_result.unwrap();
    let output: bool = match output_str{
        "true" => true,
        "false" => false,
        _ => panic!("Unexpected bool character"),
    };

    return Ok((input, Some(output)));
}

pub fn parse_string(input: ParseInput) -> Result<(ParseInput, ParseResult<String>), ParseError<'_> > {
    let start: ParseInput = input;
    let head_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput|nom::bytes::complete::tag("\"")(i);
    let content_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput|nom::bytes::complete::is_not("\"")(i);
    let end_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput|nom::bytes::complete::tag("\"")(i);
    let string_result: nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = nom::sequence::delimited(head_parser, content_parser,end_parser)(input);

    let err_handle: bool = handle_fatal_parse_error(&string_result, "invalid string")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, output): (&str, &str) = string_result.unwrap();

    return Ok((input, Some(output.to_string())));
}

pub fn parse_float(input: ParseInput) -> Result<(ParseInput, ParseResult<f64>), ParseError<'_> > {

    let start: ParseInput = input;

    let (input, integer_part) = parse_int(input)?;
    if integer_part.is_none() {
        return Ok((start, None));
    }

    let (input, dot) = parse_dot(input)?;
    if dot.is_none() {
        return Ok((start, None));
    }


    let (input, fractional_part) = parse_digit(input)?;
    if fractional_part.is_none() {
        return Ok((start, None));
    }

    let int_part: i32 = integer_part.unwrap();
    let frac_part: i32 = fractional_part.unwrap();
    let sign: f64 = int_part.signum() as f64;
    let scale: usize = power_of_ten_based_on(frac_part);
    let output:f64 = (int_part.abs() as f64 + frac_part as f64 / scale as f64) * sign;

    return Ok((input, Some(output)));

}

fn power_of_ten_based_on(digits: i32) -> usize {

    if digits == 0 {
        return 1;
    }

    let mut pow: usize = 1;
    let mut digits_abs = digits.abs();
    while digits_abs > 0 {
        pow *= 10;
        digits_abs /= 10;
    }

    return pow;

}

pub fn parse_operator(input: &str) -> Result<(ParseInput, ParseResult<Operator>), ParseError<'_> >  {
    let start: ParseInput = input;

    let parse_equal: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("==")(i);
    let parse_not_equal: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("!=")(i);
    let parse_less_than_or_equal: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("<=")(i);
    let parse_greater_than_or_equal: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag(">=")(i);
    let parse_less_than: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("<")(i);
    let parse_greater_than: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag(">")(i);
    let parse_in_keyword: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("in")(i);
    
    let operator_result: nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = nom::branch::alt((
        parse_equal,
        parse_not_equal,
        parse_less_than_or_equal,
        parse_greater_than_or_equal,
        parse_less_than,
        parse_greater_than,
        parse_in_keyword,
    ))(input);

    let err_handle: bool = handle_fatal_parse_error(&operator_result, "invalid operator")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, output): (&str, &str) =  operator_result.unwrap();

    let operator = match output {
        "==" => Operator::Equal,
        "!=" => Operator::NotEqual,
        "<=" => Operator::LessThanEqual,
        ">=" => Operator::GreaterThanEqual,
        "<" => Operator::LessThan,
        ">" => Operator::GreaterThan,
        "in" => Operator::In,
        _ => unreachable!(),
    };

    return Ok((input, Some(operator)));
}

pub fn parse_chain(input: &str) -> Result<(ParseInput, ParseResult<Chain>), ParseError<'_> >  {
    let start: ParseInput = input;

    let and_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("and")(i);
    let or_parser: fn(ParseInput) -> nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = |i: ParseInput| nom::bytes::complete::tag("or")(i);

    let chain_reuslt: nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = nom::branch::alt((
        and_parser,
        or_parser,
    ))(input);

    let err_handle: bool = handle_fatal_parse_error(&chain_reuslt, "invalid chain")?;
    if err_handle == false {
        return Ok( (start, None ) );
    }

    let (input, output): (&str, &str) =  chain_reuslt.unwrap();


    let chain:Chain = match output{
        "and" => Chain::And,
        "or" => Chain::Or,
        _ => unreachable!(),
    };

    return Ok((input, Some(chain)));
}

pub fn parse_access_key(input: &str) -> Result<(ParseInput, ParseResult<String>), ParseError<'_> >{
    let start: ParseInput = input;

    let start_char_parser: fn(char) -> bool = |c: char|  c.is_alphabetic();
    let start_result: nom::IResult<ParseInput, char, nom::error::Error<ParseInput>> = nom::character::complete::satisfy(start_char_parser)(input);
    let err_handle1: bool = handle_fatal_parse_error(&start_result, "invalid first char on variable")?;
    if err_handle1 == false {
        return Ok( (start, None ) );
    }
    let (input, start_char): (ParseInput, char) = start_result.unwrap();

    let tail_string_parser: fn(char) -> bool = |c: char| c.is_alphanumeric() || c == '_';
    let tail_result: nom::IResult<ParseInput, &str, nom::error::Error<ParseInput>> = nom::bytes::complete::take_while1(tail_string_parser)(input);
    let err_handle2: bool = handle_fatal_parse_error(&tail_result, "invalid tail strings on variable")?;
    if err_handle2 == false {
        return Ok((input, Some(start_char.to_string())))
    }
    let (input, tail_string): (ParseInput, &str) = tail_result.unwrap();

    let output: String = format!("{}{}", start_char, tail_string);
    return Ok((input, Some(output)));
}

fn parse_bracket(input: ParseInput) -> Result<(ParseInput, ParseResult<&str>), ParseError<'_>> {
    if !input.starts_with('[') {
        return Ok((input, None));
    }

    let mut stack = Vec::new();
    let mut start_pos: usize = 0;
    let mut end_pos: usize = 0;

    for (i, ch) in input.char_indices() {
        match ch {
            '[' => {
                if stack.is_empty() {
                    start_pos = i;
                }
                stack.push(ch);
            }
            ']' => {
                if stack.pop().is_none() {
                    return Err(ParseError::UnmatchedClosingBracket(input));
                }
                if stack.is_empty() {
                    end_pos = i;
                    break;
                }
            }
            _ => continue,
        }
    }

    if !stack.is_empty() {
        return Err(ParseError::UnmatchedClosingBracket(input));
    }

    return Ok((&input[end_pos+1..], Some(&input[start_pos+1..end_pos])));

}


pub fn parse_access_index(input: &str) -> Result<(ParseInput, ParseResult<AccessPath>), ParseError<'_> >{
    let start: ParseInput = input;

    let (tail, data_in_bracket_opt): (ParseInput, ParseResult<&str>) = parse_bracket(input)?;
    if data_in_bracket_opt.is_none() {
        return Ok( (start, None ) );
    }
    let data_in_bracket = data_in_bracket_opt.unwrap();

    //Access to an array must be done using a number (int) or a variable (access path).
    let (input, output): (ParseInput, ParseResult<i32>) = parse_int(data_in_bracket)?;
    if output.is_some(){
        return Ok(
            ( 
                tail, 
                Some(AccessPath::Index(IndexValue::Int(output.unwrap()))),
            )
        );
    }

    let (input, output): (ParseInput, ParseResult<Access>) = parse_access(data_in_bracket)?;
    if output.is_some(){
        return Ok(
            ( 
                tail, 
                Some(AccessPath::Index(IndexValue::Access(output.unwrap())))
            )
        );
    }


    return Err(ParseError::InvalidDataInArray(tail));

}

fn parse_continuous_indices(input: ParseInput) -> Result<(ParseInput, Vec<AccessPath>), ParseError<'_>> {
    let mut indices = Vec::new();
    let mut current_tail = input;

    loop {
        let (next_tail, index) = parse_access_index(current_tail)?;
        match index {
            Some(actual_index) => {
                indices.push(actual_index);
                current_tail = next_tail;
            },
            None => break,
        }
    }

    Ok((current_tail, indices))
}

pub fn parse_access(input: &str) -> Result<(ParseInput, ParseResult<Access>), ParseError<'_> >{
    let start: ParseInput = input;

    // When accessing a struct, the head string must be a variable name.
    // none [2222].access1
    // ok base.access1
    let (tail, base): (ParseInput, ParseResult<String>) = parse_access_key(input)?;
    if base.is_none(){
        return Ok((start, None));
    }

    let mut output: Vec<AccessPath> = Vec::new();

    // parse base[test[5]][2222]...
    // result: access([key(base), index(acess([key(test), index(int(5))])), index(int(2222))])
    let (tail, mut indices) = parse_continuous_indices(tail)?;
    output.append(&mut indices);

    let mut current_tail = tail;
    
    // parse access1.access2.access3. ...
    while let Ok((next_tail, Some(_))) = parse_dot(current_tail) {
        current_tail = next_tail;
        
        let (next_tail, path) = parse_access_key(current_tail)?;
        if path.is_none() {
            return Err(ParseError::AccessPathDotError(current_tail));
        }

        output.push(AccessPath::Key(path.unwrap()));
        current_tail = next_tail;

        let (next_tail, mut indices) = parse_continuous_indices(current_tail)?;
        output.append(&mut indices);
        current_tail = next_tail;
    }

    if output.is_empty(){
        return Ok((current_tail, Some(Access{
            base: base.unwrap(),
            path: None,
        })));
    }
    else {
        return Ok((current_tail, Some(Access{
            base: base.unwrap(),
            path: Some(output),
        })));
    }

}

pub fn parse_value(input: &str) -> Result<(ParseInput, ParseResult<Value>), ParseError<'_> > {
    let start: ParseInput = input;
    
    let (tail, output): (ParseInput, ParseResult<LiteralValue>) = parse_literal_value(input)?;
    if output.is_some(){
        return Ok(
            ( 
                tail, 
                Some(Value::Literal(output.unwrap()))
            )
        );
    }

    let (tail, output): (ParseInput, ParseResult<Access>) = parse_access(input)?;
    if output.is_some(){
        return Ok(
            ( 
                tail, 
                Some(Value::Access(output.unwrap()))
            )
        );
    }

    return Ok((start, None));
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_digit_valid_input() {
        assert_eq!(parse_digit("12345abc"), Ok(("abc", Some(12345))));
    }

    #[test]
    fn test_parse_digit_invalid_input() {
        assert_eq!( parse_digit("abc12345"), Ok(("abc12345", None)) );
    }

    #[test]
    fn test_parse_digit_empty_input() {
        assert_eq!(parse_digit(""), Ok(("", None)));
    }

    #[test]
    fn test_parse_int_positive() {
        assert_eq!(parse_int("+12345abc"), Ok(("abc", Some(12345))));
    }

    #[test]
    fn test_parse_int_negative() {
        assert_eq!(parse_int("-12345abc"), Ok(("abc", Some(-12345))));
    }

    #[test]
    fn test_parse_int_without_sign() {
        assert_eq!(parse_int("12345abc"), Ok(("abc", Some(12345))));
    }

    #[test]
    fn test_parse_int_invalid_input() {
        assert_eq!(parse_int("abc-12345"), Ok(("abc-12345", None)));
    }

    #[test]
    fn test_parse_sign_positive() {
        assert_eq!(parse_sign("+12345"), Ok(("12345", Some('+'))));
    }

    #[test]
    fn test_parse_sign_negative() {
        assert_eq!(parse_sign("-12345"), Ok(("12345", Some('-'))));
    }

    #[test]
    fn test_parse_sign_without_sign() {
        assert_eq!(parse_sign("12345"), Ok(("12345", None)));
    }

    #[test]
    fn test_parse_sign_invalid_input() {
        assert_eq!(parse_sign("abc-12345"), Ok(("abc-12345", None)));
    }

    #[test]
    fn test_parse_float_valid() {
        let (remaining, result) = parse_float("+123.456").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(result, Some(123.456));
    }


    #[test]
    fn test_parse_float_no_integer_part() {
        let (remaining, result) = parse_float(".456").unwrap();
        assert_eq!(remaining, ".456");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_float_no_fractional_part() {
        let (remaining, result) = parse_float("123.").unwrap();
        assert_eq!(remaining, "123.");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_float_invalid_input() {
        let (remaining, result) = parse_float("abc.123").unwrap();
        assert_eq!(remaining, "abc.123");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_float_empty_input() {
        let (remaining, result) = parse_float("").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_literal_value_with_float() {
        let result = parse_literal_value("3.14");
        assert_eq!(result, Ok(("", Some(LiteralValue::Float(3.14)))));
    }

    #[test]
    fn test_parse_literal_value_with_int() {
        let result = parse_literal_value("42");
        assert_eq!(result, Ok(("", Some(LiteralValue::Int(42)))));
    }

    #[test]
    fn test_parse_literal_value_with_string() {
        let result = parse_literal_value("\"hello\"");
        assert_eq!(result, Ok(("", Some(LiteralValue::String("hello".to_string())))));
    }

    #[test]
    fn test_parse_literal_value_with_bool() {
        let result = parse_literal_value("true");
        assert_eq!(result, Ok(("", Some(LiteralValue::Bool(true)))));
    }

    #[test]
    fn test_parse_literal_value_with_null() {
        let result = parse_literal_value("null");
        assert_eq!(result, Ok(("", Some(LiteralValue::Null))));
    }

    
    #[test]
    fn test_parse_access_single_key() {
        let input = "variable";
        let result = parse_access(input).unwrap();
        println!("{:?}", result);
        
        assert_eq!(
            result,
            (
                "",
                Some(Access{
                    base: "variable".to_string(),
                    path: None,
                }))
        );
        
    }
    
    #[test]
    fn test_parse_access_chained_key() {
        let input = "struct1.struct2.struct3";
        let result = parse_access(input).unwrap();
        println!("{:?}", result);
        
        assert_eq!(
            result,
            (
                "",
                Some(Access{
                    base: "struct1".to_string(),
                    path: Some(vec![
                        AccessPath::Key("struct2".to_string()),
                        AccessPath::Key("struct3".to_string())
                    ])
                })
            )
        );
        
    }
    
    #[test]
    fn test_parse_access_key_with_index() {
        let input = "array[0]";
        let result = parse_access(input).unwrap();
        assert_eq!(
            result,
            (
                "",
                Some(
                    Access{
                        base: "array".to_string(),
                        path: Some(vec![
                            AccessPath::Index(IndexValue::Int(0))
                        ])
                    }
                )
            )
        );
    }
    
    #[test]
    fn test_multiple_index_access() {
        let input = "root[0][1]";
        let result = parse_access(input).unwrap();
        println!("{:?}", result);
        assert_eq!(
            result,
            (
                "",
                Some(
                    Access{
                        base: "root".to_string(),
                        path: Some(vec![
                            AccessPath::Index(IndexValue::Int(0)),
                            AccessPath::Index(IndexValue::Int(1))
                        ])
                    }
                )
            )
        );
    }
    
    #[test]
    fn test_mixed_dot_and_index_access() {
        let input = "root.key[1]";
        let result = parse_access(input);
        println!("{:?}", result);
    }

    /*
    #[test]
    fn test_complex_dot_and_index_access_1() {
        let input = "root[1][2].key[1]";
        let expected = Some(Value::Access(vec![
            AccessPath::Key("root".to_string()), 
            AccessPath::Index(Value::Literal(LiteralValue::Int(1))), 
            AccessPath::Index(Value::Literal(LiteralValue::Int(2))), 
            AccessPath::Key("key".to_string()), 
            AccessPath::Index(Value::Literal(LiteralValue::Int(1)))
        ]));
        let result = parse_access(input);
        println!("{:?}", result);
        assert_eq!(result.map(|(_, v)| v), Ok(expected), "Failed on input: {}", input);
    }

    #[test]
    fn test_complex_dot_and_index_access_2() {
        let input = "root[0].key1.key2[2][3]";
        let expected = Some(Value::Access(vec![
            AccessPath::Key("root".to_string()), 
            AccessPath::Index(Value::Literal(LiteralValue::Int(0))), 
            AccessPath::Key("key1".to_string()), 
            AccessPath::Key("key2".to_string()), 
            AccessPath::Index(Value::Literal(LiteralValue::Int(2))),
            AccessPath::Index(Value::Literal(LiteralValue::Int(3)))
        ]));
        let result = parse_access(input);
        assert_eq!(result.map(|(_, v)| v), Ok(expected), "Failed on input: {}", input);
    }
    */
    #[test]
    fn test_struct_inside_array_access_1() {
        let input = "root[result[333]]";
        let result = parse_access(input);
        println!("{:?}", result);
    }
    /*
    #[test]
    fn test_extract_basic() {
        let input = "[hello world]!";
        let (remaining, extracted) = parse_bracket(input).unwrap();
        assert_eq!(extracted, Some("hello world"));
        assert_eq!(remaining, "!");
    }

    #[test]
    fn test_extract_without_brackets_at_start() {
        let input = "hello [world]!";
        let (remaining, extracted) = parse_bracket(input).unwrap();
        assert_eq!(extracted, None);
        assert_eq!(remaining, "hello [world]!");
    }

    #[test]
    fn test_extract_nested_brackets() {
        let input = "[hello [world]]!";
        let (remaining, extracted) = parse_bracket(input).unwrap();
        assert_eq!(extracted, Some("hello [world]"));
        assert_eq!(remaining, "!");
    }

    #[test]
    fn test_extract_unmatched_closing_bracket() {
        let input = "hello world]";
        let result = parse_bracket(input);
        assert_eq!(result, Ok(("hello world]", None)));
    }
    

    #[test]
    fn test_extract_unmatched_opening_bracket() {
        let input = "[hello world";
        let result = parse_bracket(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_without_brackets() {
        let input = "hello world!";
        let (remaining, extracted) = parse_bracket(input).unwrap();
        assert_eq!(extracted, None);
        assert_eq!(remaining, "hello world!");
    }

    #[test]
    fn test_parse_value_literal() {
        let input = "123";
        let result = parse_value(input);
        match result {
            Ok((remaining, Some(Value::Literal(LiteralValue::Int(n))))) => {
                assert_eq!(remaining, "");
                assert_eq!(n, 123);
            },
            _ => panic!("Unexpected result: {:?}", result),
        }
    }

    #[test]
    fn test_parse_value_access() {
        let input = "some_var.a.key";
        let result = parse_value(input);
        match result {
            Ok((remaining, Some(Value::Access(access_path)))) => {
                assert_eq!(remaining, "");
                assert_eq!(access_path, vec![
                    AccessPath::Key("some_var".to_string()),
                    AccessPath::Key("a".to_string()),
                    AccessPath::Key("key".to_string())
                ]);
            },
            _ => panic!("Unexpected result: {:?}", result),
        }
    }

    #[test]
    fn test_parse_value_none() {
        let input = "";
        let result = parse_value(input);
        match result {
            Ok((remaining, None)) => {
                assert_eq!(remaining, input);
            },
            _ => panic!("Unexpected result: {:?}", result),
        }
    }
    */
  
}

