use rusqlite;
use rusqlite::Params;
use std::path::Path;
use crate::config::parser::parser_type::Access;
use crate::config::parser::parser_type::AccessPath;
use crate::config::parser::parser_type::IndexValue;
use crate::config::parser::parser_type::Value;
use crate::config::conditions::Condition;
use crate::database::result::Result;
use crate::config::config::Config;
use crate::config::conditions;
use crate::config::arguments;
use crate::config::parser::parser_type;
use crate::database::error::Error;
use base64::Engine;

pub type Database = DatabaseT;
pub type Transaction<'a> = TransactionT<'a>;
pub type Statement<'a> = StatementT<'a>;

pub struct DatabaseT {
    conn: rusqlite::Connection,
}

pub struct TransactionT<'a>{
    tx: rusqlite::Transaction<'a>,
}
pub struct StatementT<'a>{
    stmt: rusqlite::Statement<'a>,
}

pub struct InsertAnalyerStatement<'a, 'b>{
    result: Statement<'a>,
    // Hashmap<analyzer name, insert stmt>
    analyzer: std::collections::HashMap<&'b str, Statement<'a>>,
}

pub struct SelectAnalyzerStatement<'a, 'b>{
    stmt: std::collections::HashMap<&'b str, (ArgumentStatement<'a>, Option<ConditionStatement<'a>>)>
}

#[derive(Clone)]
pub enum BindType {
    ResultId(i64),
}

pub enum BindRequirement {
    Required(Vec<BindType>), 
    Provided(Vec<BindType>),
}

pub enum AccessStatement<'a>{
    Stmt(Statement<'a>, BindRequirement),
    Format(String)
}

//pub enum AccessStatement<'a>{
//    // aaa.aaa[stmt].aaa -> format!($.aaa[{}].aaa.aaa) -> Format
//    // stmt = bbb.bbb[stmt2] -> format!($bbb.bbb[{}]) -> Format
//    // stmt2 = bbb.bbb -> $bbb.bbb -> stmt
//    
//    // currently, Format Parameter will not be used. becouse of Occuring error written object in json array: test1[test2[test3]]...
//    Stmt(Statement<'a>),
//    Format(String)
//}

pub struct ArgumentStatement<'a>{
    // object key, statement
    arg_stmt_list: Vec<(String, AccessStatement<'a>)>
}

pub struct ConditionStatement<'a>{
    pub cond_stmt_list: Vec<(Option<AccessStatement<'a>>, Option<AccessStatement<'a>>)>,
}

impl<'a> Statement<'a>{
    fn query_map_json<P: Params>(&mut self, param: P) -> Result<Option<serde_json::Value>>{
        //let mut rows = self.stmt.query_map(param, |row| {
        //    Ok(row.get(0)?)
        //})?;
        
        //if let Some(row_result) = rows.next() {
        //    Ok(Some(row_result?))
        //} else {
        //    Ok(None)
        //}
        let mut rows = self.stmt.query_map(param, |row| {
            match row.get_ref(0)? {
                rusqlite::types::ValueRef::Text(text) => Ok(serde_json::Value::String(String::from_utf8_lossy(text).into_owned())),
                rusqlite::types::ValueRef::Integer(int) => Ok(serde_json::Value::Number(int.into())),
                rusqlite::types::ValueRef::Real(float) => Ok(serde_json::Value::Number(serde_json::Number::from_f64(float).unwrap())),
                rusqlite::types::ValueRef::Blob(blob) => {
                    let encoder = base64::engine::general_purpose::STANDARD;
                    Ok(serde_json::Value::String(encoder.encode(blob)))
                },
                rusqlite::types::ValueRef::Null => Ok(serde_json::Value::Null),
            }
        })?;
        
        if let Some(row_result) = rows.next() {
            Ok(Some(row_result?))
        } else {
            Ok(None)
        }
    }


    fn execute_insert<P: Params>(&mut self, param: P) -> Result<(), Error> {
        self.stmt.execute(param)?;
        Ok(())
    }
}

impl<'a, 'b> SelectAnalyzerStatement<'a, 'b>{
    pub fn get_stmt(&mut self, analyzer_name: &str) -> Result<&mut (ArgumentStatement<'a>, Option<ConditionStatement<'a>>)>{
        self.stmt.get_mut(analyzer_name).ok_or_else(|| Error::NotAnalyzerNameInDataBase(analyzer_name.to_string()))
    }


    pub fn process_analyzer_stmt(&mut self, analyzer_name: &str, conditions: Option<&Vec<conditions::Condition>>) -> Result<()> {
        let (argument_stmt, cond_stmt_opt) = self.stmt.get_mut(analyzer_name).ok_or_else(|| Error::NotAnalyzerNameInDataBase(analyzer_name.to_string()))?;
        match (conditions, cond_stmt_opt) {
            (Some(conditions), Some(cond_stmt)) => {
                if conditions.len() != cond_stmt.cond_stmt_list.len() {
                    return Err(Error::DiffCondAndCondStmt());
                }
                for (cond, stmt) in conditions.iter().zip(&mut cond_stmt.cond_stmt_list) {
                    //cond.is_match_condition(stmt)?; 
                    stmt.is_match_condition(cond)?;
                }
            }
            (None, None) => {}
            _ => return Err(Error::DiffCondAndCondStmt()),
        }
        
        Ok(())
    }
}

pub trait IsMatchCodition{
    fn is_match_condition(&mut self, config: &Condition) -> Result<bool, Error>;
}

impl<'a> IsMatchCodition for (Option<AccessStatement<'a>>, Option<AccessStatement<'a>>) {
    fn is_match_condition(&mut self, cond: &Condition) -> Result<bool, Error>{
    //pub fn is_match_condition<'a>(&self, select_stmt: &'a mut (Option<AccessStatement<'a>>, Option<AccessStatement<'a>>)) -> Result<bool, Error>{
        let mut left_json: serde_json::Value;
        let mut right_json: serde_json::Value;
        match cond {
            Condition { 
                left: Value::Literal(left_lit), 
                op, 
                right: Value::Literal(right_lit), 
                chain 
            } => 
            {
                if self.0.is_some() || self.1.is_some(){
                    return Err(Error::NoAccessSatement());
                }
                left_json = left_lit.into();
                right_json = right_lit.into();
                Ok( json_compare(&left_json, &op, &right_json)? )
            },
            Condition { 
                left: parser_type::Value::Literal(left_lit), 
                op, 
                right: parser_type::Value::Access(right_acc), 
                chain 
            } => 
            {
                if self.0.is_some() || self.1.is_none(){
                    return Err(Error::NoAccessSatement());
                }
                left_json = left_lit.into();
                //right_json = match select_stmt.1.as_ref().ok_or(Error::NoAccessSatement())?.to_json()?{
                //    Some(json) => json,
                //    None => return Ok(false)
                //};
                match &mut self.1 {
                    Some(stmt) => {
                        match stmt.to_json()?{
                            Some(json) => {
                                right_json = json;
                                Ok( json_compare(&left_json, &op, &right_json)? )
                            }
                            None => {
                                return Ok(false);
                            }
                        }
                    }
                    None => {
                        return Err(Error::NoAccessSatement());
                    }
                }
            },
            Condition { 
                left: parser_type::Value::Access(left_acc), 
                op, 
                right: parser_type::Value::Literal(right_lit), 
                chain 
            } => 
            {
                right_json = right_lit.into();
                match &mut self.0 {
                    Some(stmt) => {
                        match stmt.to_json()?{
                            Some(json) => {
                                left_json = json;
                                Ok( json_compare(&left_json, &op, &right_json)? )
                            }
                            None => {
                                return Ok(false);
                            }
                        }
                    }
                    None => {
                        return Err(Error::NoAccessSatement());
                    }
                }
            },
            Condition { 
                left: parser_type::Value::Access(left_acc), 
                op, 
                right: parser_type::Value::Access(right_acc), 
                chain 
            } => 
            {
                if self.0.is_none() || self.1.is_none(){
                    return Err(Error::NoAccessSatement());
                }
                match (&mut self.0 ,&mut self.1) {
                    (Some(stmt_left), Some(stmt_right)) => {
                        match (stmt_left.to_json()?, stmt_right.to_json()?){
                            (Some(json_left), Some(json_right)) => {
                                left_json = json_left;
                                right_json = json_right;
                                Ok( json_compare(&left_json, &op, &right_json)? )
                            }
                            _ => {
                                return Ok(false);
                            }
                        }
                    }
                    _ => {
                        return Err(Error::NoAccessSatement());
                    }
                }
            },
        }
    }
}

impl<'a> ConditionStatement<'a> {
    pub fn is_match_condition(&mut self, conditions: &Vec<conditions::Condition>) -> Result<bool, Error> {
        if conditions.len() != self.cond_stmt_list.len() {
            return Err(Error::DiffCondAndCondStmt());
        }

        for (cond, mut stmt) in conditions.iter().zip(self.cond_stmt_list.iter_mut()) {
            if !stmt.is_match_condition(&cond)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn set_placeholder(&mut self, result_id_for_place_holder: i64) -> Result<()>{
        for (left_opt, right_opt) in self.cond_stmt_list.iter_mut() {
            left_opt.as_mut().map(|left| left.set_placeholder(result_id_for_place_holder)).transpose()?;
            right_opt.as_mut().map(|right| right.set_placeholder(result_id_for_place_holder)).transpose()?;
        }
        Ok(())
    }

}

impl<'a> ArgumentStatement<'a> {
    pub fn get_argument(&mut self) -> Result<serde_json::Value, Error> {
        let mut rst_arg = serde_json::Value::Object(serde_json::Map::new());

        for (object_key, stmt) in self.arg_stmt_list.iter_mut() {
            let arg: serde_json::Value = match stmt.to_json()? {
                Some(json) => json,
                None => serde_json::Value::Null,
            };

            match rst_arg.as_object_mut() {
                Some(obj) => {
                    obj.insert(object_key.clone(), arg);
                }
                None => {
                    return Err(Error::CanNotMutableObject());
                }
            }
        }
        Ok(rst_arg)
    }

    pub fn set_placeholder(&mut self, result_id_for_place_holder: i64) -> Result<()>{
        for (object_key, stmt) in self.arg_stmt_list.iter_mut() {
            stmt.set_placeholder(result_id_for_place_holder)?;
        }
        Ok(())
    }
}


impl<'a, 'b> InsertAnalyerStatement<'a, 'b> {
    pub fn insert_path(&mut self, path: &std::path::Path) -> Result<bool, Error>{
        self.result.execute_insert([path.to_string_lossy().as_ref()])?;
        Ok(true)
    }

    pub fn insert_analyzer(&mut self, analyzer_name: &str, result_id: i64, value: serde_json::Value) -> Result<bool, Error>{
        match self.analyzer.get_mut(analyzer_name) {
            Some(stmt) => {
                stmt.execute_insert(rusqlite::params![result_id, value])?;
                Ok(true)
            }
            None => {
                Err(Error::NoAnalyzerName())
            }
        }

    }
}



impl<'a> Transaction<'a> {


    pub fn start_transaction(db: &'a mut Database) -> Result<Self> {
        let tx: Transaction = db.start_transaction()?;
        Ok(tx) 
    }

    pub fn end_transaction(self) -> Result<()> {
        self.tx.commit()?;
        Ok(())
    }

    fn prepare(&'a self, sql: &str) ->  Result<Statement<'a>> {
        let stmt: rusqlite::Statement  = self.tx.prepare(sql)?;
        let rst: Statement = Statement { stmt: stmt };
        return Ok(rst);
    }

    pub fn create_insert_result_stmt(&'a self) -> Result<Statement<'a>> {
        return self.prepare(&"INSERT INTO result (path) VALUES (?1)");
    }

    pub fn create_insert_analyzer_stmt<'b>(&'a self, config: &'b Config) -> Result<std::collections::HashMap<&'b str, Statement<'a>>> {
        let mut analyzer_list = std::collections::HashMap::new();
        for analyzer in &config.analyzer {
            analyzer_list.insert(
                analyzer.name.as_str(),
                self.prepare(format!("INSERT INTO {} (result_id, value) VALUES (?1, ?2)", analyzer.name).as_str())?
            );
        }
        Ok(analyzer_list)
    }

    pub fn insert_stmt<'b>(&'a self, config: &'b Config) -> Result<InsertAnalyerStatement<'a, 'b>>{
        Ok(InsertAnalyerStatement{
            result: self.create_insert_result_stmt()?,
            analyzer: self.create_insert_analyzer_stmt(config)?,
        })
    }

    pub fn select_stmt<'b>(&'a self, config: &'b Config) -> Result<SelectAnalyzerStatement<'a, 'b>>{
        let mut rst: std::collections::HashMap<&'b str, (ArgumentStatement<'a>, Option<ConditionStatement<'a>>)>
            = std::collections::HashMap::new();
        for analyzer in &config.analyzer{
            rst.insert(&analyzer.name, (self.argument_stmt(&analyzer.arguments)?,self.condition_stmt(&analyzer.conditions)? ));
        }
        return Ok(SelectAnalyzerStatement{stmt: rst});
    }

    pub fn argument_stmt<'b>(&'a self, opt_arg_list: &'b Option<Vec<arguments::Argument>>) -> Result<ArgumentStatement<'a>>{
        let mut arg_stmt: Vec<(String, AccessStatement)> = Vec::new();
        let value_stmt: AccessStatement = Access{base: "path".to_string(), path: None}.generate_stmt(self)?;
        arg_stmt.push(("filename".to_string() , value_stmt));
        match opt_arg_list{
            Some(arg_list) =>  {
                let mut cnt: u64 = 1;
                for arg in arg_list {
                    let value_stmt: AccessStatement = arg.generate_stmt(self)?;
                    arg_stmt.push((format!("argument{}", cnt) , value_stmt));
                    cnt += 1;
                }
            }
            None => {
            }
        }
        Ok(ArgumentStatement{arg_stmt_list: arg_stmt})
    }

    pub fn condition_stmt<'b>(&'a self, opt_vec_cond: &'b Option<Vec<conditions::Condition>>) -> Result<Option<ConditionStatement<'a>>>{
        let mut cond_stmt: Vec<(Option<AccessStatement>, Option<AccessStatement>)> = Vec::new();
        match opt_vec_cond{
            Some(vec_cond) =>  {
                for cond in vec_cond {
                    let value_stmt: (Option<AccessStatement>, Option<AccessStatement>) = cond.generate_stmt_condition(self)?;
                    cond_stmt.push(value_stmt);
                }
            }
            None => {
                return Ok(None);
            }
        }
        Ok(Some(ConditionStatement{cond_stmt_list: cond_stmt}))
    }

}

impl DatabaseT{

    pub fn open(path: &Path) -> Result<Self> {
        let conn: rusqlite::Connection = rusqlite::Connection::open(path)?;
        let db: Database = Database { conn: conn };
        return Ok(db);
    }

    
    pub fn start_transaction<'a>(&'a mut self) -> Result<Transaction<'a>> {
        let tx = self.conn.transaction()?;
        let rst: Transaction = Transaction { tx: tx };
        Ok(rst) 
    }

    pub fn create_result_table(&self) -> Result<()> {
        self.conn.execute( &"CREATE TABLE result (
                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                path TEXT
                            )", 
                        [] )?;
        Ok(())
    }
    
    pub fn create_analyzer_table(&self, config: &Config) -> Result<()> {
        for analyzer in &config.analyzer {
            self.conn.execute( &format!("CREATE TABLE {} (
                                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                                            result_id INTEGER,
                                            value JSON
                                            )",
                                    &analyzer.name
                                ), 
                                [])?;
        }
        Ok(())
    }
    
}

impl<'a> AccessStatement<'a> {
    pub fn to_json(&mut self) -> Result<Option<serde_json::Value>, Error> {
        match self {
            AccessStatement::Stmt(stmt, bind_req) => {
                let bind_values: Vec<&dyn rusqlite::ToSql> = match bind_req {
                    BindRequirement::Required(_) => {
                        return Err(Error::BindRequired());
                    },
                    BindRequirement::Provided(values) => {
                        values.iter().map(|val| match val {
                            BindType::ResultId(i) => i as &dyn rusqlite::ToSql,
                        }).collect()
                    }
                };

                stmt.query_map_json(&bind_values[..])
            },
            AccessStatement::Format(_) => {
                Err(Error::UnimplementedError())
            }
        }
    }

    pub fn set_placeholder(&mut self, result_id_for_placeholder: i64) -> Result<(), Error> {
        if let AccessStatement::Stmt(_, bind_req) = self {
            match bind_req {
                BindRequirement::Required(types) => {
                    for btype in types.iter_mut() {
                        match btype {
                            BindType::ResultId(_) => {
                                *btype = BindType::ResultId(result_id_for_placeholder);
                            },
                        }
                    }
                    *bind_req = BindRequirement::Provided(types.clone());
                },
                BindRequirement::Provided(values) => {
                    for value in values.iter_mut() {
                        match value {
                            BindType::ResultId(_) => {
                                *value = BindType::ResultId(result_id_for_placeholder);
                            },
                        }
                    }
                }
            }
        }

        Ok(())
    }

}

trait GenerateStmt {
    fn generate_stmt<'a>(&self, tx: &'a Transaction<'a>) -> Result<AccessStatement<'a>>;
}

impl GenerateStmt for arguments::Argument {
    fn generate_stmt<'a>(&self, tx: &'a Transaction<'a>) -> Result<AccessStatement<'a>>{
        return self.value.generate_stmt(tx);
    }
}

trait GetStatementCondition {
    fn generate_stmt_condition<'a>(&self, tx: &'a Transaction<'a>) -> Result<(Option<AccessStatement<'a>>, Option<AccessStatement<'a>>)>;
}

impl GetStatementCondition for conditions::Condition{
    fn generate_stmt_condition<'a>(&self, tx: &'a Transaction<'a>) -> Result<(Option<AccessStatement<'a>>, Option<AccessStatement<'a>>)> {
        return Ok((self.left.generate_stmt_condition(tx)?, self.right.generate_stmt_condition(tx)?));
    }
}

trait GetStatementValue {
    fn generate_stmt_condition<'a>(&self, tx: &'a Transaction<'a>) -> Result<Option<AccessStatement<'a>>>;
}


impl GetStatementValue for parser_type::Value{
    fn generate_stmt_condition<'a>(&self, tx: &'a Transaction<'a>) -> Result<Option<AccessStatement<'a>>> {
        match self{
            Value::Literal(_) => Ok(None),
            Value::Access(ac) => Ok(Some(ac.generate_stmt(tx)?))
        }
    }
}

// path -> get current path
// path[integer] -> select path where id = ?
// path["aaaa"] -> error
// feature -> path["aaaaaa"].analyzer_name -> select value->'$' from analyzer_name where path = "aaaaaa"
// analyzer_name.aaa.bbb. -> select value->'$aaa.bbb.' from analyzer_name where id = current
// analyzer_name[] -> error

//-------------------------------------
// there is a sql injection!!
// take measures against it when parsing configuration strings (e.g., parsing Access variables).
//------------------------------------
impl GenerateStmt for parser_type::Access {
    fn generate_stmt<'a>(&self, tx: &'a Transaction<'a>) -> Result<AccessStatement<'a>>{
        match self {
            // path
            parser_type::Access{ base: base , path: None} if base == "path" => {
                let sql = "SELECT path FROM result WHERE id = ?1";
                let latest_path_stmt: Statement = tx.prepare(&sql)?;
                return Ok(AccessStatement::Stmt(latest_path_stmt, BindRequirement::Required(vec!(BindType::ResultId(0)))));
            },

            parser_type::Access{base: base, path: path} if base == "path" => {
                return Err(Error::UnimplementedError());
            }

            parser_type::Access{ base: base , path: None} if base == "pathlist" => {
                let sql = "SELECT path FROM result;";
                let path_list_stmt: Statement = tx.prepare(&sql)?;
                return Ok(AccessStatement::Stmt(path_list_stmt, BindRequirement::Required(vec!())));
            },

            parser_type::Access{ base: base , path: path} if base == "pathlist" => {
                return Err(Error::PathListDoesNotHaveAcess());
            },
            

            // base = analyzer_name
            parser_type::Access{base: analyzer_name, path: opt_json_path} => {
                // json data in an analyzer_name table is stored in `value` column
                let mut db_json_operator: String = format!("");

                // first, check if the first path is not contained accesspath::index 
                // analyzer_name dose not have array
                // for exsample `analyzer_name[5]` is error
                db_json_operator = match opt_json_path {
                    Some(json_path) => {
                        if let Some(path) = json_path.first() {
                            match path {
                                AccessPath::Index(_) => return Err(Error::AnalyzerNameDoesNotHaveAnArray()),
                                AccessPath::Key(key) => {
                                    // the json access operator for sqlite is written like `culumn_name->>$.access_key` at the beginning.
                                    db_json_operator = db_json_operator + &format!(".{}", key);
                                },
                            }
                        }
                        db_json_operator
                    },
                    // if only analyzer_name. get latest json data in analyzer_name table
                    None => {
                        let sql = &format!("SELECT analyzer.value->>'$'
                                                                FROM {} AS analyzer 
                                                                JOIN result ON analyzer.result_id = result.id
                                                                WHERE result.id = ?1;", analyzer_name);
                        let latest_analyzer_stmt: Statement = tx.prepare(sql)?;
                        return Ok(AccessStatement::Stmt(latest_analyzer_stmt, BindRequirement::Required(vec!(BindType::ResultId(0)))));
                        }
                };

                db_json_operator = match opt_json_path {
                    Some(json_path) => {
                        for p in json_path.iter().skip(1) {
                            match p {
                                AccessPath::Key(key) => {
                                    db_json_operator = db_json_operator + &format!(".{}", key);
                                },
                                AccessPath::Index(idx) => {
                                    match idx {
                                        IndexValue::Access(_) => {
                                            // in the featuer, IndexValue::Access will be not error. allow object in json array: test1[test2], test1[test2[test3]]
                                            return Err(Error::JsonArrayDoesNotHaveOtherThanInt());
                                        },
                                            // currently, access for json array is only integer.
                                        IndexValue::Int(i) => {
                                            db_json_operator = db_json_operator + &format!("[{}]", i);
                                        },
                                    }
                                },
                            }
                        }
                        db_json_operator
                    }
                    None => panic!("expected_error!")
                };

                let sql = &format!("SELECT analyzer.value->>'${}'
                                                        FROM {} AS analyzer 
                                                        JOIN result ON analyzer.result_id = result.id
                                                        WHERE result.id = ?1", db_json_operator, analyzer_name);
                let latest_analyzer_stmt: Statement = tx.prepare(sql)?;
                return Ok(AccessStatement::Stmt(latest_analyzer_stmt, BindRequirement::Required(vec!(BindType::ResultId(0)))));
            }
        }
    }
}

pub fn json_compare(left: &serde_json::Value, op: &parser_type::Operator, right: &serde_json::Value) -> Result<bool>{
    match op {
        parser_type::Operator::Equal => Ok(left == right),
        parser_type::Operator::NotEqual => Ok(left != right),
        parser_type::Operator::GreaterThan | parser_type::Operator::GreaterThanEqual | parser_type::Operator::LessThan | parser_type::Operator::LessThanEqual => {
            if let (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) = (left, right) {
                if let (Some(l_int), Some(r_int)) = (n1.as_i64(), n2.as_i64()) {
                    match op {
                        parser_type::Operator::GreaterThan => Ok(l_int > r_int),
                        parser_type::Operator::LessThan => Ok(l_int < r_int),
                        parser_type::Operator::GreaterThanEqual => Ok(l_int >= r_int),
                        parser_type::Operator::LessThanEqual => Ok(l_int <= r_int),
                        _ => Err(Error::ComparisonErrorTypeMismatch()),
                    }
                } 
                else if let (Some(l_float), Some(r_float)) = (n1.as_f64(), n2.as_f64()) {
                    match op {
                        parser_type::Operator::GreaterThan => Ok(l_float > r_float),
                        parser_type::Operator::LessThan => Ok(l_float < r_float),
                        parser_type::Operator::GreaterThanEqual => Ok(l_float >= r_float),
                        parser_type::Operator::LessThanEqual => Ok(l_float <= r_float),
                        _ => Err(Error::ComparisonErrorTypeMismatch()),
                    }
                }
                else {
                    Err(Error::ComparisonErrorTypeMismatch())
                }
            } else {
                Err(Error::ComparisonErrorTypeMismatch())
            }
        },
        parser_type::Operator::In => {
            match (left, right) {
                (serde_json::Value::String(l_str), serde_json::Value::String(r_str)) => Ok(l_str.contains(r_str)),
                (serde_json::Value::Object(l_obj), serde_json::Value::String(r_key)) => Ok(l_obj.contains_key(r_key)),
                (serde_json::Value::Array(l_arr), r_val) => Ok(l_arr.contains(r_val)),
                _ => Err(Error::ComparisonErrorTypeMismatch()),
            }
        }
    }
}


