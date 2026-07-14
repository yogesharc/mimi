use std::{
    collections::HashMap,
    io::Read,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::tools::{Property, ToolDefinition};

pub fn bash(args: Value) -> Result<Value> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .context("missing command to run")?;

    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn bash command: {command}"))?;

    let mut output = String::new();
    if let Some(mut o) = child.stdout.take() {
        o.read_to_string(&mut output)
            .context("failed to read stdout")?;
    }

    let mut stderr = String::new();
    if let Some(mut e) = child.stderr.take() {
        e.read_to_string(&mut stderr)
            .context("failed to read stderr")?;
    }

    let status = child.wait().context("failed to wait for bash")?;
    if !status.success() {
        let code = status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".to_string());
        bail!("bash exited with {code}\nstdout:\n{output}\nstderr:\n{stderr}");
    }

    Ok(serde_json::json!({"output": output}))
}

pub fn def_bash() -> ToolDefinition {
    let name = "bash".to_string();
    let description =
        "Run a bash command. Pass a single shell command string; it is executed with bash -c."
            .to_string();
    let strict = true;
    let cmd_prop = Property {
        description: "bash command to run (e.g. \"ls -la\" or \"echo hello\")".to_string(),
        ..Default::default()
    };

    let properties = HashMap::from([("command".to_string(), cmd_prop)]);
    let required = Some(vec!["command".to_string()]);

    ToolDefinition::new(name, description, strict, properties, required)
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn run_bash() {
        let args = serde_json::to_value(json!({
            "command": "echo 'hello world'"
        }))
        .unwrap();

        let output = bash(args).unwrap();
        let output = output.get("output").and_then(|c| c.as_str()).unwrap();

        assert_eq!(output.trim(), "hello world");
    }
}
