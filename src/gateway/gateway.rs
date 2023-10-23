use crate::config::config;
use crate::gateway;
use crate::gateway::error::Error;
use crate::database::database;


use once_cell::sync::Lazy;
use std::sync::Mutex;
static RESULT_ID: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(1));

fn analyze_callback(
    base: &std::path::Path, 
    relative_path: &std::path::Path, 
    absolute_path: &std::path::Path, 
    script_dir: &std::path::Path,
    config: &config::Config,
    insert_stmt: &mut database::InsertAnalyerStatement, 
    select_stmt: &mut database::SelectAnalyzerStatement,
) -> Result<(), Error> {
    insert_stmt.insert_path(&relative_path)?;
    let mut result_id_guard = RESULT_ID.lock().unwrap();
    let result_id = *result_id_guard;
    *result_id_guard += 1;
    for analyzer in &config.analyzer {
        {
            let (arg_stmt, cond_stmt) = select_stmt.get_stmt(&analyzer.name)?;
            match (analyzer.conditions.as_ref(), cond_stmt) {
                (Some(conditions), Some(stmt)) => {
                    stmt.set_placeholder(result_id)?;
                    if stmt.is_match_condition(&conditions)? == true {
                        arg_stmt.set_placeholder(result_id)?;
                        let mut args: serde_json::Value = arg_stmt.get_argument()?;
                        if let Some(obj) = args.as_object_mut() {
                            obj.insert("relative_path".to_string(), serde_json::json!(base.display().to_string()));
                            obj.insert("absolute_path".to_string(), serde_json::json!(absolute_path.display().to_string()));
                        }
                        let result: serde_json::Value = gateway::dispatcher::dispatcher::execute_analyzer(script_dir, &analyzer.name, &analyzer.extension, &args)?;
                        insert_stmt.insert_analyzer(&analyzer.name, result_id, result)?;
                    }
                    else {
                        continue;
                    }

                }
                (None, None) => {
                    arg_stmt.set_placeholder(result_id)?;
                    let mut args: serde_json::Value = arg_stmt.get_argument()?;
                    if let Some(obj) = args.as_object_mut() {
                        obj.insert("relative_path".to_string(), serde_json::json!(base.display().to_string()));
                        obj.insert("absolute_path".to_string(), serde_json::json!(absolute_path.display().to_string()));
                    }
                    let result: serde_json::Value = gateway::dispatcher::dispatcher::execute_analyzer(script_dir, &analyzer.name, &analyzer.extension, &args)?;
                    insert_stmt.insert_analyzer(&analyzer.name, result_id, result)?;
                }
                _ => return Err(Error::DiffCondAndCondStmt()),
            }
        } 
    }
    

    Ok(())
}

pub fn analyze(
    firmware_root_directory: &std::path::Path, 
    script_directory: &std::path::Path, 
    config_file: &std::path::Path,
    database_file: &std::path::Path,
) -> Result<(), Error> 
{

    let canonical_path = std::fs::canonicalize(firmware_root_directory)?;
    let abs_path: &std::path::Path = canonical_path.as_path();
    let config: config::Config = config::Config::load(config_file)?;
    let mut db: database::Database = database::Database::open(database_file)?;
    {
        db.create_result_table()?;
        db.create_analyzer_table(&config)?;
        let transaction: database::Transaction = database::Transaction::start_transaction(&mut db)?;
        {
            let mut insert_stmt: database::InsertAnalyerStatement = transaction.insert_stmt(&config)?;
            let mut select_stmt: database::SelectAnalyzerStatement = transaction.select_stmt(&config)?;
            traverse_dir(abs_path, abs_path, script_directory, &config, &mut insert_stmt, &mut select_stmt, &analyze_callback)?;
        }
        transaction.end_transaction()?;
    }
    Ok(())
}

fn traverse_dir<F>(
    base: &std::path::Path,
    current: &std::path::Path,
    script_dir: &std::path::Path,
    config: &config::Config,
    insert_stmt: &mut database::InsertAnalyerStatement,
    select_stmt: &mut database::SelectAnalyzerStatement,
    callback: &F,
) -> Result<(), Error>
where
    F: Fn(
        &std::path::Path, 
        &std::path::Path, 
        &std::path::Path, 
        &std::path::Path, 
        &config::Config,
        &mut database::InsertAnalyerStatement, 
        &mut database::SelectAnalyzerStatement
    ) -> Result<(), Error>,
{
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(base).unwrap_or(&path);

        callback(base, relative_path, &path, script_dir, config, insert_stmt, select_stmt)?;

        if path.is_dir() {
            traverse_dir(base, &path, script_dir, config, insert_stmt, select_stmt, callback)?;
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs;

    #[test]
    fn test_analyze() -> Result<(), Error> {
        let firmware_root_directory = Path::new("./src");
        let script_directory = Path::new("./test_file/script");
        let config_file = Path::new("./test_file/config2.toml");
        let database_file = Path::new("path_to_test_db");

        fs::create_dir_all(firmware_root_directory)?;

        let result = analyze(firmware_root_directory, script_directory, config_file, database_file);
        println!("{:?}", result);
        //assert!(ouesult.is_ok());

        Ok(())
    }
}
