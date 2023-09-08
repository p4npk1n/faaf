extern crate serde_json;
pub struct AnalysisResult{
    result: serde_json::Value,
}

impl AnalysisResult {
    pub fn new() -> Self {
        AnalysisResult {
            result: serde_json::Value::Null,
        }
    }
}