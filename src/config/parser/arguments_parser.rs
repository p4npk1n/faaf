use crate::config::parser::base_parser;
use crate::config::arguments;


#[derive(Debug, PartialEq)]
pub enum ParseArgumentError<'a>{
    BaseParseError(base_parser::ParseError<'a>),
    NoneValue(base_parser::ParseInput<'a>, &'a str),
    InvalidValue(base_parser::ParseInput<'a>, &'a str)
}

impl<'a> From<base_parser::ParseError<'a>> for ParseArgumentError<'a> {
    fn from(err: base_parser::ParseError<'a>) -> ParseArgumentError<'a> {
        ParseArgumentError::BaseParseError(err)
    }
}

impl<'a> std::fmt::Display for ParseArgumentError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseArgumentError::BaseParseError(err) => write!(f, "{}", err),
            ParseArgumentError::NoneValue(input, add_info) => write!(f, "None Value Error {} ,  error info :{:?}", input, add_info),
            ParseArgumentError::InvalidValue(input, add_info) => write!(f, "Invaild Value: input: {},  error info :{:?}", input, add_info),
        }
    }
}

impl<'a> std::error::Error for ParseArgumentError<'a> {}


pub fn parse_argument(input: base_parser::ParseInput) -> Result<arguments::Argument, ParseArgumentError<'_>> {
    let start: base_parser::ParseInput = input;

    // [multispace] [value] [multispace]
    let (remaining, space) = base_parser::parse_whitespace(input)?;
    // [value] [multispace]
    let (remaining, value) = base_parser::parse_access(remaining)?;
    if remaining.is_empty() && !value.is_none(){
        // 111 EOF
        return Ok(arguments::Argument {
            value: value.unwrap(),
        });
    }
    if !remaining.is_empty() && value.is_none(){
        // \n\n\t
        return Err(ParseArgumentError::NoneValue(remaining, "not argument"));
    }
    if remaining.is_empty() && value.is_none(){
        // empty
        return Err(ParseArgumentError::NoneValue(remaining, "The variable is empty"));
    }


    // [multispace] 
    let (remaining, space) = base_parser::parse_whitespace(remaining)?;
    // I don't need it because I know the strings is not empty in the code above.
    //if result.is_none() && remaining.is_empty(){
    //}
    if !space.is_none() && remaining.is_empty(){
        // 111 \t\n
        return Ok(arguments::Argument {
            value: value.unwrap(),
        });
    }
    if space.is_none() && !remaining.is_empty(){
        // for exsample if `111aaa` is parsed as int type, remainig = aaa, result = Value::Int(111).
        return Err(ParseArgumentError::InvalidValue(remaining, "Unexpected data exists in value suffix"));
    }

    //  1111 gagdsfgsdfsd
    return Err(ParseArgumentError::InvalidValue(remaining, "Unexpected data exists in value suffix"));
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_argument() {
        let input = "aaa";
        let result = parse_argument(input).unwrap();
        println!("{:?}", result);
        // assert_eq!(result.value, base::Value::A(base::LiteralValue::Int(123)));

        let input = " bbb \t\n";
        let result = parse_argument(input).unwrap();
        println!("{:?}", result);
        //assert_eq!(result.value, base::Value::Literal(base::LiteralValue::Int(123)));

        let input = "bbbb";
        let result = parse_argument(input);
        println!("{:?}", result);
        //assert!(result.is_err());

        let input = "1111";
        let result = parse_argument(input);
        println!("{:?}", result);
        //assert!(result.is_err());

        let input = " \t\n";
        let result = parse_argument(input);
        println!("{:?}", result);
        //assert!(result.is_err());
    }
}
