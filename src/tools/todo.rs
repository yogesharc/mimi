use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::{fs, io::AsyncWriteExt};

use crate::tools::{Property, ToolDefinition};

pub async fn create_todo_list(session_id: &str, args: Value) -> Result<Value> {
    let contents = args
        .get("contents")
        .and_then(|c| c.as_str())
        .context("missing contents")?;

    let dir = mimi_dir().await?;
    let path = dir.join(format!("todo_{}.md", session_id));

    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .await?;

    file.write_all(contents.as_bytes()).await?;

    Ok(serde_json::json!({ "todo_created": format!("todo_{}.md", session_id) }))
}

pub fn def_create_todo_list() -> ToolDefinition {
    let name = "create_todo_list".to_string();
    let description =
        "Create a markdown todo list for the current session. Use for multi-step tasks to track progress."
            .to_string();
    let strict = true;

    let contents_property = Property {
        description: "Markdown todo list contents (e.g. checklist items with - [ ] / - [x] status)"
            .to_string(),
        ..Default::default()
    };

    let properties = HashMap::from([("contents".to_string(), contents_property)]);
    let required = Some(vec!["contents".to_string()]);

    ToolDefinition::new(name, description, strict, properties, required)
}

async fn mimi_dir() -> Result<PathBuf> {
    let path = std::env::current_dir()?.join(".mimi");
    fs::create_dir_all(&path)
        .await
        .context("failed to create mimi dir in cwd")?;

    Ok(path)
}
