use serde::de::Deserializer;
use serde::Deserialize;
use crate::config::parser;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub analyzer: Vec<Analyzer>,
}

#[derive(Debug)]
pub struct Analyzer {
    name: String,
    extension: String,
    arguments: Option<Vec<parser::arguments::Argument>>,
    dependencies: Option<Vec<String>>,
    conditions: Option<Vec<parser::conditions::Condition>>,
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

        let inner = InnerAnalyzer::deserialize(deserializer)?;

        let arguments = inner.arguments.map(|args| {
            args.into_iter()
                .filter_map(|arg| {
                    parser::arguments::parse_argument(&arg).ok().map(|(_, argument)| argument)
                })
                .collect()
        });

        let conditions = inner.conditions.map(|conditions_str| {
            conditions_str.split("\n")
                .filter_map(|line| {
                    parser::conditions::parse_condition(line.trim()).ok().map(|(_, condition)| condition)
                })
                .collect()
        });

        Ok(Analyzer {
            name: inner.name,
            extension: inner.extension,
            arguments,
            dependencies: inner.dependencies,
            conditions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_deserialize_config() {

        let mut file = File::open("test_file/config.toml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();


        let config: Config = toml::from_str(&contents).unwrap();

        for analyzer in &config.analyzer {
            println!("{:#?}", analyzer);
        }

    }
}
