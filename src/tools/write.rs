use crate::tools::{Property, ToolDefinition};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use uuid::Uuid;

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    truncate: bool,
    content: String,
}

pub fn write_to_file(args: Value) -> Result<Value, String> {
    let parsed_args = serde_json::from_value::<WriteArgs>(args).map_err(|e| e.to_string())?;

    let path = Path::new(&parsed_args.path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(parsed_args.truncate)
        .open(path)
        .map_err(|e| e.to_string())?;

    file.write_all(parsed_args.content.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success:": "ok"}))
}

pub fn make_a_copy(path: &str) -> Result<(), String> {
    let id = Uuid::now_v7();
    let to = format!("{id}-{path}");

    fs::copy(path, to).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_file(path: &str) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| e.to_string())?;

    Ok(())
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

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::tools::read::read_file;

    use super::*;

    #[test]
    fn write() {
        let path = "test/write_here.txt";
        let content = "hello world";
        let overwrite = true;
        let args = serde_json::to_value(json!({
            "path": path,
            "content": content,
            "overwrite": overwrite
        }))
        .unwrap();

        let _ = write_to_file(args);

        let path = serde_json::to_value(json!({"path": "test/write_here.txt"})).unwrap();
        let output = read_file(path).unwrap();
        let content = output.get("content").and_then(|c| c.as_str()).unwrap();

        assert_eq!(content, "hello world")
    }
}
