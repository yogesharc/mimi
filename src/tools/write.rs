use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use serde_json::Value;

use crate::tools::{Property, ToolDefinition};

pub fn write_to_file(args: Value) -> Result<Value, String> {
    let path_str = args
        .get("path")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "missing path to write file".to_string())?;

    let path = Path::new(path_str);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let truncate = args
        .get("overwrite")
        .and_then(|c| c.as_bool())
        .ok_or_else(|| "missing whether to overwrite or not".to_string())?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(path)
        .map_err(|e| e.to_string())?;

    let contents = args
        .get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "missing content".to_string())?;

    file.write_all(contents.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success:": "ok"}))
}

pub fn def_write_to_file() -> ToolDefinition {
    let name = "write_to_file".to_string();
    let description = "Creates or writes contents to an existing file".to_string();
    let strict = true;

    let path_property = Property {
        description: "Provide a file path to write to".to_string(),
        ..Default::default()
    };

    let content_property = Property {
        description: "Provide the content to write".to_string(),
        ..Default::default()
    };

    let overwrite_property = Property {
        r#type: "boolean".to_string(),
        description: "Define whether to overwrite an existing file or not".to_string(),
        items: None,
        property_enum: None,
    };

    let properties = HashMap::from([
        ("path".to_string(), path_property),
        ("content".to_string(), content_property),
        ("overwrite".to_string(), overwrite_property),
    ]);

    let required = Some(vec![
        "path".to_string(),
        "content".to_string(),
        "overwrite".to_string(),
    ]);
    ToolDefinition::new(name, description, strict, properties, required)
}
