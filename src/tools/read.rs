use serde_json::Value;
use std::{collections::HashMap, env, fs};

use crate::tools::{Property, ToolDefinition};

pub fn read_file(path: Value) -> Result<Value, String> {
    let dir = env::current_dir().map_err(|e| e.to_string())?;

    let path = path
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| "missing path".to_string())?;

    let full_path = dir.join(path);

    let contents = fs::read_to_string(full_path).map_err(|e| e.to_string())?;

    println!("READ FILE RESULT: {contents}");

    Ok(serde_json::json!({"contents": contents}))
}

pub fn def_read_file() -> ToolDefinition {
    let name = "read_file".to_string();
    let description = "Read and returns full file contents".to_string();
    let strict = true;

    let path_property = Property {
        description: "Provide a file path to write to".to_string(),
        ..Default::default()
    };

    let properties = HashMap::from([("path".to_string(), path_property)]);

    let required = Some(vec!["path".to_string()]);

    ToolDefinition::new(name, description, strict, properties, required)
}
