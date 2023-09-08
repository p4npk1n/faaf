use crate::config;
use crate::analysis_result;

/*
pub fn analyze(firmware_root_directory: &std::path::Path, analyzer_directory: &std::path::Path, 
    analyzer_config: &config::Config) -> Result<analysis_result::AnalysisResult, Error>{


}
*/

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    StripPrefixError(std::path::StripPrefixError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IO Error: {}", err),
            Error::StripPrefixError(err) => write!(f, "Strip Prefix Error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        return Error::IoError(err);
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(err: std::path::StripPrefixError) -> Error {
        return Error::StripPrefixError(err);
    }
}

pub fn get_all_entries(firmware_root_directory: &std::path::Path) -> Result<Vec<std::path::PathBuf>, Error> {
    let mut to_explore: std::collections::VecDeque<std::path::PathBuf> = std::collections::VecDeque::new();
    let mut output: Vec<std::path::PathBuf> = Vec::new();
    
    to_explore.push_back(firmware_root_directory.to_path_buf());

    while let Some(path) = to_explore.pop_front() {
        let relative_path: &std::path::Path = match path.strip_prefix(firmware_root_directory){
            Ok(relative_path) => relative_path,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        output.push(std::path::Path::new("/").join(relative_path));

        if path.is_dir() {
            let entries: std::fs::ReadDir = match std::fs::read_dir(&path) {
                Ok(result) => result,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };

            for entry_rst in entries {
                let entry: std::fs::DirEntry = match entry_rst {
                    Ok(entry) => entry,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
                to_explore.push_back(entry.path());
            }
        }
    }

    return Ok(output);
}


pub fn analyze(firmware_root_directory: &std::path::Path, config_file: &std::path::Path, analyzer_directory: &std::path::Path, output: &std::path::Path) -> bool {
    // Currently, this function does not return any Err values.
    let entries: Vec<std::path::PathBuf> = get_all_entries(firmware_root_directory).unwrap();
    let config: Box<config::Config> = match config::Config::load(config_file) {
        Ok(box_config) => box_config,
        Err(err) => return false,
    };
    let db = database::Database::new();

    let result: analysis_result::AnalysisResult = analysis_result::AnalysisResult::new();

    for entry in entries{
        for conf_analyzer in &config.analyzer{
            if (conf_analyzer.is_match_condition(&result) == true) {
                dispatcher::execute_analyzer(&entry, &conf_analyzer, &result)
            }
            else {
                continue;
            }
        }
    }

    db.write(&result);

    return true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_all_entries() {
        let test_dir = PathBuf::from("./");
        let entries = get_all_entries(&test_dir).unwrap();
        for entry in &entries {
            println!("{:?}", entry);
        }
    }

}
