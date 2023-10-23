use serde::Deserialize;
use crate::config::analyzer;
use crate::config::error::Error;
use toml;


#[derive(Deserialize, Debug)]
pub struct Config {
    pub analyzer: Vec<analyzer::Analyzer>,
}


impl Config {
    pub fn load(config_file: &std::path::Path) -> Result<Self, Error>{
        let data: String = std::fs::read_to_string(config_file)?;
        let config: Config = toml::from_str(&data)?;
        Ok(config)
    }
}
