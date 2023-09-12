extern crate serde;
extern crate toml;
pub mod parser;

#[derive(serde::Deserialize, Debug)]
pub struct Analyzer {
    name: String,
    extension: String,
    arguments: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
    conditions: Option<String>
}

/*
assert_eq!(config.analyzer[0].name, "analyzer1");
assert_eq!(config.analyzer[0].extension, "ext1");
assert_eq!(config.analyzer[1].name, "analyzer2");
assert_eq!(config.analyzer[1].extension, "ext2");
assert_eq!(config.analyzer[1].arguments.as_ref().unwrap()[0], "arg1");
*/
