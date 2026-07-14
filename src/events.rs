use crate::parser::AgentEventItem;

use anyhow::{Context, Result, bail};

use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

// static APP_PATH: &str = ".mimi";

fn get_app_dir() -> Result<PathBuf> {
    let app_dir;

    if let Some(dir) = std::env::var_os("HOME") {
        app_dir = PathBuf::from(dir).join(".mimi");
    } else {
        bail!("could not find the home directory");
    }

    fs::create_dir_all(&app_dir)
        .with_context(|| format!("failed to create app directory {}", app_dir.display()))?;

    Ok(app_dir)
}

pub async fn append_events(
    session_id: &String,
    events: &Vec<AgentEventItem>,
    create_new: bool,
) -> Result<()> {
    if session_id.is_empty() {
        bail!("session ID is required");
    }

    let sessions_dir = get_app_dir()?.join("sessions");

    fs::create_dir_all(&sessions_dir).with_context(|| {
        format!(
            "failed to create sessions directory {}",
            sessions_dir.display()
        )
    })?;

    let file_name = format!("{session_id}.jsonl");
    let path = sessions_dir.join(file_name);

    let file = OpenOptions::new()
        .create_new(create_new)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open session log {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    for item in events {
        let json =
            serde_json::to_string_pretty(&item).context("failed to serialize session event")?;
        writeln!(writer, "{json}")
            .with_context(|| format!("failed to write session log {}", path.display()))?;
    }

    writer
        .flush()
        .with_context(|| format!("failed to flush session log {}", path.display()))?;

    Ok(())
}
