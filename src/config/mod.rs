extern crate serde;
extern crate toml;
pub mod analyzer;

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub analyzer: Vec<analyzer::Analyzer>,
}

impl Config {
    pub fn load(filename: &std::path::Path) -> Result<Box<Config>, Error>{
        let contents = std::fs::read_to_string(filename.to_path_buf())?;
        let config: Config = toml::from_str(&contents)?;
        // sort_dependencies()
        return Ok(Box::new(config));
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TomlError(toml::de::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IO Error: {}", err),
            Error::TomlError(err) => write!(f, "TOML Error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        return Error::IoError(err);
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        return Error::TomlError(err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_valid_config() {

        let data = r#"
            [[analyzer]]
            name = "analyzer1"
            extension = "ext1"

            [[analyzer]]
            name = "analyzer2"
            extension = "ext2"
            arguments = ["arg1", "arg2"]
        "#;


        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("config.toml");

        {
            let mut file = File::create(&file_path).expect("Failed to create temp file");
            file.write_all(data.as_bytes()).expect("Failed to write to temp file");
        }

        let config = Config::load(&file_path).expect("Failed to load config");
        assert_eq!(config.analyzer.len(), 2);
    }

    #[test]
    fn test_load_invalid_toml() {
        let data = r#"
            [[analyzer]
            name = "missing closing bracket"
        "#;

        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("invalid_config.toml");

        {
            let mut file = File::create(&file_path).expect("Failed to create temp file");
            file.write_all(data.as_bytes()).expect("Failed to write to temp file");
        }

        let result = Config::load(&file_path);
        assert!(matches!(result, Err(Error::TomlError(_))));
    }

}
