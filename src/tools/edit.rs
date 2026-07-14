use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use anyhow::{Context, Result, bail};

use crate::tools::{Property, ToolDefinition};

#[derive(Debug, Serialize, Deserialize)]
struct Edit {
    path: String,
    r#type: EditType,
    old_content: Option<String>,
    new_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum EditType {
    Replace,
    Delete,
    Append,
}

pub fn edit_file(args: Value) -> Result<Value> {
    let edit =
        serde_json::from_value::<Edit>(args).context("failed to parse edit tool arguments")?;
    let path = Path::new(&edit.path);

    let mut content =
        read_to_string(path).with_context(|| format!("failed to read file {}", path.display()))?;

    match edit.r#type {
        EditType::Replace | EditType::Delete => {
            let old_content = edit.old_content.as_deref().unwrap_or("");
            let new_content = edit.new_content.as_deref().unwrap_or("");

            let updated_content = content.replacen(old_content, new_content, 1);

            fs::write(path, updated_content)
                .with_context(|| format!("failed to write file {}", path.display()))?;
        }
        EditType::Append => {
            let old_content = edit.old_content.as_deref().unwrap_or("");
            let new_content = edit.new_content.as_deref().unwrap_or("");

            let Some(idx) = content.find(old_content) else {
                bail!("old_content not found in file");
            };
            content.insert_str(idx + old_content.len(), new_content);

            fs::write(path, content)
                .with_context(|| format!("failed to write file {}", path.display()))?;
        }
    }

    Ok(serde_json::json!({"success:": "ok"}))
}

pub fn def_edit_file() -> ToolDefinition {
    let name = "edit_file".to_string();
    let description = "Find and replace, delete, or append content in a file".to_string();
    let strict = true;

    let path_property = Property {
        description: "Path to the file to edit".to_string(),
        ..Default::default()
    };

    let type_property = Property {
        description:
            "Edit operation: 'Replace' to substitute text, 'Delete' to remove text, 'Append' to insert after a match"
                .to_string(),
        property_enum: Some(vec![
            "Replace".to_string(),
            "Delete".to_string(),
            "Append".to_string(),
        ]),
        ..Default::default()
    };

    let old_content_property = Property {
        description:
            "Existing text to locate in the file. For Append, new_content is inserted after this match"
                .to_string(),
        ..Default::default()
    };

    let new_content_property = Property {
        description: "Replacement or appended text (Replace/Append). Use empty string for Delete"
            .to_string(),
        ..Default::default()
    };

    let properties = HashMap::from([
        ("path".to_string(), path_property),
        ("type".to_string(), type_property),
        ("old_content".to_string(), old_content_property),
        ("new_content".to_string(), new_content_property),
    ]);

    let required = Some(vec![
        "path".to_string(),
        "type".to_string(),
        "old_content".to_string(),
        "new_content".to_string(),
    ]);

    ToolDefinition::new(name, description, strict, properties, required)
}
