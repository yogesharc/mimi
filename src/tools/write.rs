use crate::tools::{Property, ToolDefinition};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
// use uuid::Uuid;

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    overwrite: bool,
    content: String,
}

pub fn write_to_file(args: Value) -> Result<Value> {
    let parsed_args = serde_json::from_value::<WriteArgs>(args)
        .context("failed to parse write tool arguments")?;

    let path = Path::new(&parsed_args.path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let mut options = OpenOptions::new();
    options.write(true);

    if parsed_args.overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }

    let mut file = options
        .open(path)
        .with_context(|| format!("failed to open file {}", path.display()))?;

    file.write_all(parsed_args.content.as_bytes())
        .with_context(|| format!("failed to write file {}", path.display()))?;

    Ok(serde_json::json!({"success:": "ok"}))
}

// pub fn make_a_copy(path: &str) -> Result<()> {
//     let id = Uuid::now_v7();
//     let to = format!("{id}-{path}");

//     fs::copy(path, &to).with_context(|| format!("failed to copy {path} to {to}"))?;

//     Ok(())
// }

// pub fn delete_file(path: &str) -> Result<()> {
//     fs::remove_file(path).with_context(|| format!("failed to delete file {path}"))?;

//     Ok(())
// }

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
    fn overwrites_an_existing_file() {
        let path = "test/write_here.txt";
        let content = "hello world";
        let overwrite = true;
        let args = serde_json::to_value(json!({
            "path": path,
            "content": content,
            "overwrite": overwrite
        }))
        .unwrap();

        write_to_file(args).unwrap();

        let path = serde_json::to_value(json!({"path": "test/write_here.txt"})).unwrap();
        let output = read_file(path).unwrap();
        let content = output.get("content").and_then(|c| c.as_str()).unwrap();

        assert_eq!(content, "hello world")
    }

    #[test]
    fn overwrite_false_preserves_an_existing_file() {
        let path = "test/do_not_overwrite.txt";
        fs::write(path, "existing content").unwrap();
        let args = json!({
            "path": path,
            "content": "new",
            "overwrite": false
        });

        let result = write_to_file(args);

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "existing content");
    }
}
