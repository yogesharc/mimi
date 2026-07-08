use std::{
    collections::HashMap,
    io::Read,
    process::{Command, Stdio},
};

use serde_json::Value;

use crate::tools::{Property, ToolDefinition};

pub fn shell(args: Value) -> Result<Value, String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing command to run")?;

    let arguments: Vec<&str> = args
        .get("arguments")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "errrr".to_string())?
        .iter()
        .map(|a| {
            a.as_str()
                .ok_or_else(|| "each argument must be a string".to_string())
        })
        .collect::<Result<Vec<&str>, String>>()?;

    let mut child = Command::new(command)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut output = String::new();
    if let Some(mut o) = child.stdout.take() {
        o.read_to_string(&mut output).map_err(|e| e.to_string())?;
    }

    let _ = child.wait().map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"output": output}))
}

pub fn def_shell() -> ToolDefinition {
    let name = "shell".to_string();
    let description = "dasjkdjsakdjsa".to_string();
    let strict = true;
    let cmd_prop = Property {
        description: "command to run".to_string(),
        property_enum: Some(vec![
            "sh".to_string(),
            "zsh".to_string(),
            "bash".to_string(),
            "ls".to_string(),
        ]),
        ..Default::default()
    };
    let args_prop = Property {
        r#type: "array".to_string(),
        description: "Define the arguments to pass to the command".to_string(),
        items: Some(Box::new(Property::default())),
        property_enum: None,
    };

    let properties = HashMap::from([
        ("command".to_string(), cmd_prop),
        ("arguments".to_string(), args_prop),
    ]);

    let required = Some(vec!["command".to_string(), "arguments".to_string()]);

    ToolDefinition::new(name, description, strict, properties, required)
}
