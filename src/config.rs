extern crate serde;
extern crate toml;


#[derive(serde::Deserialize, Debug)]
pub struct Analyzer {
    name: String,
    extension: String,
    arguments: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
    conditions: Option<String>
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    analyzer: Vec<Analyzer>,
}

impl Config {
    pub fn load(filename: &str) -> Result<Config, Error>{
        let contents = std::fs::read_to_string(filename)?;
        let config: Config = toml::from_str(&contents)?;
        return Ok(config);
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
        // テスト用のTOMLデータ
        let data = r#"
            [[analyzer]]
            name = "analyzer1"
            extension = "ext1"

            [[analyzer]]
            name = "analyzer2"
            extension = "ext2"
            arguments = ["arg1", "arg2"]
        "#;

        // 一時ディレクトリを作成
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("config.toml");

        // データを一時ファイルに書き込み
        {
            let mut file = File::create(&file_path).expect("Failed to create temp file");
            file.write_all(data.as_bytes()).expect("Failed to write to temp file");
        }

        // load関数を呼び出してテスト
        let config = Config::load(file_path.to_str().unwrap()).expect("Failed to load config");
        assert_eq!(config.analyzer.len(), 2);
        assert_eq!(config.analyzer[0].name, "analyzer1");
        assert_eq!(config.analyzer[0].extension, "ext1");
        assert_eq!(config.analyzer[1].name, "analyzer2");
        assert_eq!(config.analyzer[1].extension, "ext2");
        assert_eq!(config.analyzer[1].arguments.as_ref().unwrap()[0], "arg1");
    }

    #[test]
    fn test_load_invalid_toml() {
        // 不正なTOMLデータ
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

        let result = Config::load(file_path.to_str().unwrap());
        assert!(matches!(result, Err(Error::TomlError(_))));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Config::load("nonexistent.toml");
        assert!(matches!(result, Err(Error::IoError(_))));
    }
}
