extern crate nom;
use crate::config::parser::base;

#[derive(Debug, PartialEq)]
pub struct Condition {
    left: base::Value,
    op: base::Operator,
    right: base::Value,
    chain: Option<base::Chain>,
}



pub fn parse_condition(i: &str) -> nom::IResult<&str, Condition> {
    
    let (i, (_, left, _, op, _, right, _, chain, _)) =  nom::sequence::tuple((
        nom::character::complete::multispace0,
        base::parse_value,
        nom::character::complete::multispace0,
        base::parse_operator,
        nom::character::complete::multispace0,
        base::parse_value,
        nom::character::complete::multispace0,
        base::parse_chain,
        nom::character::complete::multispace0,
    ))(i)?;
    Ok((i, Condition { left, op, right, chain }))
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
                    base::Value::Literal(base::LiteralValue::String(s)) => println!("{}", s),
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
