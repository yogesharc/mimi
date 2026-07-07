use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
pub mod file_search;
use file_search::Search;
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
    // #[serde(rename = "enum")]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // property_enum: Option<Vec<String>>,
}

impl Default for Property {
    fn default() -> Self {
        Self {
            r#type: "string".to_string(),
            description: String::new(),
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

// =========== Tools Definition and Functions =============

pub fn get_date(_args: Value) -> Result<Value, String> {
    //this fn does not use args, its just to kinda half bake the args thing
    Ok(serde_json::json!({"date": "5 Jul, 2026"}))
}

pub fn def_get_date() -> ToolDefinition {
    let name = String::from("get_date");
    let description = String::from("Get relative date");
    let strict = true;
    let mut properties = HashMap::new();
    let property = Property {
        r#type: "string".to_string(),
        description: "Specify names e.g today, yesterday".to_string(),
    };
    properties.insert("relative".to_string(), property);
    let required = Some(vec![String::from("relative")]);
    ToolDefinition::new(name, description, strict, properties, required)
}

pub fn get_weather(_args: Value) -> Result<Value, String> {
    Ok(serde_json::json!("Sunny"))
}

pub fn def_get_weather() -> ToolDefinition {
    let name = String::from("get_weather");
    let description = String::from("Get dummy weather for a place");
    let strict = true;
    let mut properties = HashMap::new();
    let property = Property {
        r#type: "string".to_string(),
        description: "Place to get weather for, e.g. Kathmandu".to_string(),
    };
    properties.insert("place".to_string(), property);
    let required = Some(vec![String::from("place")]);
    ToolDefinition::new(name, description, strict, properties, required)
}

// =========== System Tools =============

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemTools {
    GetDate,
    GetWeather,
    SearchFiles,
}

impl SystemTools {
    pub fn all() -> Vec<Self> {
        vec![Self::GetDate, Self::GetWeather, Self::SearchFiles]
    }

    pub fn variant_from_name(name: &str) -> Option<Self> {
        match name {
            "get_date" => Some(Self::GetDate),
            "get_weather" => Some(Self::GetWeather),
            "search_files" => Some(Self::SearchFiles),
            _ => None,
        }
    }

    pub fn definition(&self) -> ToolDefinition {
        match self {
            SystemTools::GetDate => def_get_date(),
            SystemTools::GetWeather => def_get_weather(),
            SystemTools::SearchFiles => Search::def_search_files(),
        }
    }

    pub fn execute(&self, arguments: &str, search: Option<&Search>) -> Result<Value, String> {
        let args: Value = serde_json::from_str(arguments).map_err(|e| e.to_string())?;

        match self {
            SystemTools::GetDate => get_date(args),
            SystemTools::GetWeather => get_weather(args),
            SystemTools::SearchFiles => {
                let search = search.ok_or_else(|| "search state is required".to_string())?;
                search.search_files(args)
            }
        }
    }
}
