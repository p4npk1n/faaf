use crate::config::arguments;
use crate::config::conditions;
use serde::de::Deserializer;
use serde::Deserialize;
use crate::config::parser::{arguments_parser, conditions_parser};

#[derive(Debug)]
pub struct Analyzer {
    pub name: String,
    pub extension: String,
    pub arguments: Option<Vec<arguments::Argument>>,
    pub dependencies: Option<Vec<String>>,
    pub conditions: Option<Vec<conditions::Condition>>,
}

impl<'de> Deserialize<'de> for Analyzer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InnerAnalyzer {
            name: String,
            extension: String,
            arguments: Option<Vec<String>>,
            dependencies: Option<Vec<String>>,
            conditions: Option<String>,
        }

        let inner: InnerAnalyzer = InnerAnalyzer::deserialize(deserializer)?;

        let arguments: Option<Vec<arguments::Argument>> = if let Some(arg_strs) = inner.arguments {
            let mut parsed_arguments = Vec::new();
            for arg_str in arg_strs {
                match arguments_parser::parse_argument(&arg_str) {
                    Ok(arg) => parsed_arguments.push(arg),
                    Err(err) => {
                        return Err(serde::de::Error::custom(format!(
                            "Failed to parse argument: {}, error: {:?}",
                            arg_str, err
                        )))
                    }
                }
            }
            Some(parsed_arguments)
        }
        else {
            None
        };

        let conditions: Option<Vec<conditions::Condition>> = if let Some(condition_statement) = inner.conditions {
            let mut parsed_conditions = Vec::new();
            for cond_str in condition_statement.lines() {
                match conditions_parser::parse_condition(&cond_str) {
                    Ok(cond) => parsed_conditions.push(cond),
                    Err(err) => {
                        return Err(serde::de::Error::custom(format!(
                            "Failed to parse conditions: {}, error: {:?}",
                            cond_str, err
                        )))
                    }
                }
            }
            Some(parsed_conditions)
        }
        else {
            None
        };

        Ok(Analyzer {
            name: inner.name,
            extension: inner.extension,
            arguments: arguments,
            dependencies: inner.dependencies,
            conditions: conditions,
        })
    }
}
