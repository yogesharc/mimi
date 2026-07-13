use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
pub mod edit;
pub mod file_search;
pub mod read;
pub mod shell;
pub mod write;
use file_search::Search;

use crate::tools::{
    edit::{def_edit_file, edit_file},
    read::{def_read_file, read_file},
    shell::{def_shell, shell},
    write::{def_write_to_file, write_to_file},
};
// =========== Tools Structs and Enums =============

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    r#type: String,
    name: String,
    description: String,
    strict: bool,
    parameters: Option<Parameters>,
}

#[derive(Debug, Serialize)]
struct Parameters {
    r#type: String,
    properties: HashMap<String, Property>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct Property {
    r#type: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<Property>>,
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    property_enum: Option<Vec<String>>,
}

impl Default for Property {
    fn default() -> Self {
        Self {
            r#type: "string".to_string(),
            description: String::new(),
            items: None,
            property_enum: None,
        }
    }
}

impl ToolDefinition {
    pub fn new(
        name: String,
        description: String,
        strict: bool,
        properties: HashMap<String, Property>,
        required: Option<Vec<String>>,
    ) -> Self {
        ToolDefinition {
            r#type: "function".to_string(),
            name,
            description,
            strict,
            parameters: Some(Parameters {
                r#type: "object".to_string(),
                properties,
                required,
            }),
        }
    }
}

// =========== System Tools =============

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemTools {
    SearchFiles,
    SearchContent,
    ReadFile,
    WriteFile,
    EditFile,
    Shell,
}

impl SystemTools {
    pub fn all() -> Vec<Self> {
        vec![
            Self::SearchFiles,
            Self::SearchContent,
            Self::ReadFile,
            Self::WriteFile,
            Self::EditFile,
            Self::Shell,
        ]
    }

    pub fn variant_from_name(name: &str) -> Option<Self> {
        match name {
            "read_file" => Some(Self::ReadFile),
            "write_to_file" => Some(Self::WriteFile),
            "edit_file" => Some(Self::EditFile),
            "shell" => Some(Self::Shell),
            "search_files" => Some(Self::SearchFiles),
            "search_content" => Some(Self::SearchContent),
            _ => None,
        }
    }

    pub fn definition(&self) -> ToolDefinition {
        match self {
            SystemTools::ReadFile => def_read_file(),
            SystemTools::WriteFile => def_write_to_file(),
            SystemTools::EditFile => def_edit_file(),
            SystemTools::Shell => def_shell(),
            SystemTools::SearchFiles => Search::def_search_files(),
            SystemTools::SearchContent => Search::def_search_content(),
        }
    }

    pub fn execute(&self, arguments: &str, search: Option<&Search>) -> Result<Value, String> {
        let args: Value = serde_json::from_str(arguments).map_err(|e| e.to_string())?;

        match self {
            SystemTools::ReadFile => read_file(args),
            SystemTools::WriteFile => write_to_file(args),
            SystemTools::EditFile => edit_file(args).map_err(|e| e.to_string()),
            SystemTools::Shell => shell(args),
            SystemTools::SearchFiles => {
                let search = search.ok_or_else(|| "search state is required".to_string())?;
                search.search_files(args)
            }
            SystemTools::SearchContent => {
                let search = search.ok_or_else(|| "search state is required".to_string())?;
                search.search_content(args)
            }
        }
    }
}
