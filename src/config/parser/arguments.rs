extern crate nom;
use crate::config::parser::base;

#[derive(Debug, PartialEq)]
pub struct Argument {
    pub var: base::Value,
}

pub fn parse_argument(i: &str) -> nom::IResult<&str, Argument> {
    
    let (i, (_, var, _)) =  nom::sequence::tuple((
        nom::character::complete::multispace0,
        base::parse_value,
        nom::character::complete::multispace0,
    ))(i)?;
    Ok((i, Argument {var}))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_argument() {
        let input = "  some_value  ";
        match parse_argument(input) {
            Ok((remaining, arg)) => {
                println!("Parsed: {:?}, Remaining: {:?}", arg, remaining);
                assert_eq!(remaining, "");
            },
            Err(e) => panic!("Failed to parse with error: {:?}", e),
        }
    }
}
