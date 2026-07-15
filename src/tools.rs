use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
pub mod approval;
pub mod bash;
pub mod edit;
pub mod file_search;
pub mod read;
pub mod write;
use file_search::Search;

use crate::tools::{
    // approval::approve_tool,
    bash::{bash, def_bash},
    edit::{def_edit_file, edit_file},
    read::{def_read_file, read_file},
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
    Bash,
}

impl SystemTools {
    pub fn all() -> Vec<Self> {
        vec![
            Self::SearchFiles,
            Self::SearchContent,
            Self::ReadFile,
            Self::WriteFile,
            Self::EditFile,
            Self::Bash,
        ]
    }

    pub fn variant_from_name(name: &str) -> Option<Self> {
        match name {
            "read_file" => Some(Self::ReadFile),
            "write_to_file" => Some(Self::WriteFile),
            "edit_file" => Some(Self::EditFile),
            "bash" => Some(Self::Bash),
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
            SystemTools::Bash => def_bash(),
            SystemTools::SearchFiles => Search::def_search_files(),
            SystemTools::SearchContent => Search::def_search_content(),
        }
    }

    pub fn execute(&self, arguments: &str, search: Option<&Search>) -> Result<Value> {
        let args: Value =
            serde_json::from_str(arguments).context("failed to parse tool arguments as JSON")?;

        match self {
            SystemTools::ReadFile => read_file(args),
            SystemTools::WriteFile => {
                write_to_file(args)

                // let overwrite = args
                //     .get("overwrite")
                //     .and_then(|v| v.as_bool())
                //     .context("overwrite is missing")?;
                // let mut approved = true;
                // let path;

                // if overwrite {
                //     path = args
                //         .get("path")
                //         .and_then(|v| v.as_str())
                //         .context("path is missing")?;

                //     println!("ASKING APPROVAL -- WRITE -- {path}");

                //     approved = approve_tool();
                // }
                // if approved {
                //     write_to_file(args)
                // } else {
                //     Ok(serde_json::json!({"status": "tool call rejected by user"}))
                // }
            }
            SystemTools::EditFile => {
                edit_file(args)

                // let path = args
                //     .get("path")
                //     .and_then(|v| v.as_str())
                //     .context("path is missing")?;

                // println!("ASKING APPROVAL -- EDIT -- {path}");

                // let approved = approve_tool();

                // if approved {
                //     edit_file(args)
                // } else {
                //     Ok(serde_json::json!({"status": "tool call rejected by user"}))
                // }
            }
            SystemTools::Bash => {
                bash(args)
                // let cmd = args
                //     .get("command")
                //     .and_then(|v| v.as_str())
                //     .context("command is missing")?;

                // println!("ASKING APPROVAL: {cmd}");
                // let approved = approve_tool();

                // if approved {
                //     bash(args)
                // } else {
                //     Ok(serde_json::json!({"status": "tool call rejected by user"}))
                // }
            }
            SystemTools::SearchFiles => {
                let search = search.context("search state is required")?;
                search.search_files(args)
            }
            SystemTools::SearchContent => {
                let search = search.context("search state is required")?;
                search.search_content(args)
            }
        }
    }
}
