use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{collections::HashMap, env, fs};

use crate::tools::{Property, ToolDefinition};

pub fn read_file(path: Value) -> Result<Value> {
    let dir = env::current_dir()
        .context("failed to get current directory")?
        .canonicalize()?;

    let path = path
        .get("path")
        .and_then(|p| p.as_str())
        .context("missing path")?;

    let resolved = &dir.join(path).canonicalize()?;

    if !resolved.starts_with(&dir) {
        bail!("path is outside the current working directory")
    }

    let content = fs::read_to_string(&resolved)
        .with_context(|| format!("failed to read file {}", resolved.display()))?;

    // println!("READ FILE RESULT: {content}");

    Ok(serde_json::json!({"content": content}))
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

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn read_test() {
        let path = serde_json::to_value(json!({"path": "test/read_this.txt"})).unwrap();
        let output = read_file(path).unwrap();
        let content = output.get("content").and_then(|c| c.as_str()).unwrap();

        assert_eq!(content, "hey")
    }
}
