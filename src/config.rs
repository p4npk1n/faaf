extern crate serde;
extern crate toml;

use serde::Deserialize;
use std::fs;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Analyzer {
    name: String,
    extension: String,
    dependencies: Option<Vec<String>>,
    conditions: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct Config {
    analyzer: Vec<Analyzer>,
}

pub fn print_config(config: &Config) {
    println!("Config:");

    for (index, analyzer) in config.analyzer.iter().enumerate() {
        println!("  Analyzer {}:", index + 1);
        println!("    Name: {}", analyzer.name);
        println!("    Extension: {}", analyzer.extension);

        if let Some(ref deps) = analyzer.dependencies {
            println!("    Dependencies: {:?}", deps);
        } else {
            println!("    Dependencies: None");
        }

        if let Some(ref cond) = analyzer.conditions {
            println!("  Conditions:");
            for line in cond.lines() {
                println!("    {}", line);
            }
        }
        println!();
    }
}


pub fn load(filename: &str) -> Result<Config, Box<dyn Error>>{
    let contents = fs::read_to_string(filename)?;
    let config: Config = toml::from_str(&contents)?;
    return Ok(config);
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    const SAMPLE_TOML: &str = r#"
    [[analyzer]]
    name = "basic"
    extension = "so"

    [[analyzer]]
    name = "ldd"
    extension = "sh"
    dependencies = ["basic"]
    conditions = """
        basic.mime == \"application/x-pie-executable\" and
        basic.size > 5000
    """

    [[analyzer]]
    name = "ghidra"
    extension = "sh"
    dependencies = ["basic", "ldd", "cveinfo"]
    "#;


    #[test]
    fn test_load() -> Result<(), Box<dyn Error>> {

        let temp_file = "temp.toml";
        fs::write(temp_file, SAMPLE_TOML)?;

        let config: Config = load(temp_file)?;

        assert_eq!(config.analyzer.len(), 3);

        assert_eq!(config.analyzer[0].name, "basic");
        assert_eq!(config.analyzer[0].extension, "so");
        assert_eq!(config.analyzer[0].dependencies, None);
        assert_eq!(config.analyzer[0].conditions, None);

        assert_eq!(config.analyzer[1].name, "ldd");
        assert_eq!(config.analyzer[1].extension, "sh");
        assert_eq!(config.analyzer[1].dependencies, Some(vec!["basic".to_string()]));
        assert_eq!(config.analyzer[1].conditions, Some("        basic.mime == \"application/x-pie-executable\" and\n        basic.size > 5000\n    ".to_string()));

        assert_eq!(config.analyzer[2].name, "ghidra");
        assert_eq!(config.analyzer[2].extension, "sh");
        assert_eq!(config.analyzer[2].dependencies, Some(vec!["basic".to_string(), "ldd".to_string(), "cveinfo".to_string()]));
        assert_eq!(config.analyzer[2].conditions, None);

        print_config(&config);

        fs::remove_file(temp_file)?;

        Ok(())
    }
}