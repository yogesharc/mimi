use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
};

use serde::Deserialize;
use serde_json::Value;

use anyhow::{Context, Result, bail};

use crate::tools::{Property, ToolDefinition};

#[derive(Debug, Deserialize)]
struct Edit {
    path: String,
    r#type: EditType,
    old_content: String,
    new_content: String,
}

#[derive(Debug, Deserialize)]
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

    let match_index = find_unique_match(&content, &edit.old_content)?;
    let match_end = match_index + edit.old_content.len();

    match edit.r#type {
        EditType::Replace => {
            content.replace_range(match_index..match_end, &edit.new_content);
        }
        EditType::Delete => content.replace_range(match_index..match_end, ""),
        EditType::Append => {
            content.insert_str(match_end, &edit.new_content);
        }
    }

    fs::write(path, content).with_context(|| format!("failed to write file {}", path.display()))?;

    Ok(serde_json::json!({"success:": "ok"}))
}

fn find_unique_match(content: &str, old_content: &str) -> Result<usize> {
    if old_content.is_empty() {
        bail!("old_content must not be empty");
    }

    let mut matches = content
        .char_indices()
        .filter_map(|(index, _)| content[index..].starts_with(old_content).then_some(index));
    let Some(index) = matches.next() else {
        bail!("old_content not found in file");
    };

    if matches.next().is_some() {
        bail!("old_content matched more than once; provide more surrounding content");
    }

    Ok(index)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_old_content_returns_an_error_without_changing_the_file() {
        let path = "test/edit_missing.txt";
        fs::write(path, "original content").unwrap();
        let args = json!({
            "path": path,
            "type": "Replace",
            "old_content": "not present",
            "new_content": "replacement"
        });

        let result = edit_file(args);

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "original content");
    }

    #[test]
    fn ambiguous_old_content_returns_an_error() {
        let path = "test/edit_ambiguous.txt";
        fs::write(path, "same and same").unwrap();
        let args = json!({
            "path": path,
            "type": "Delete",
            "old_content": "same",
            "new_content": ""
        });

        let result = edit_file(args);

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "same and same");
    }

    #[test]
    fn overlapping_old_content_returns_an_error() {
        let path = "test/edit_overlapping.txt";
        fs::write(path, "aaa").unwrap();
        let args = json!({
            "path": path,
            "type": "Replace",
            "old_content": "aa",
            "new_content": "b"
        });

        let result = edit_file(args);

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "aaa");
    }
}
