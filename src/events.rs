use crate::parser::AgentEventItem;

use anyhow::{Context, Result, bail};
use tokio::{
    fs,
    io::{AsyncWriteExt, BufWriter},
};

use std::path::PathBuf;

// static APP_PATH: &str = ".mimi";

async fn get_app_dir() -> Result<PathBuf> {
    let app_dir;

    if let Some(dir) = std::env::var_os("HOME") {
        app_dir = PathBuf::from(dir).join(".mimi");
    } else {
        bail!("could not find the home directory");
    }

    fs::create_dir_all(&app_dir)
        .await
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

    let sessions_dir = get_app_dir().await?.join("sessions");

    fs::create_dir_all(&sessions_dir).await.with_context(|| {
        format!(
            "failed to create sessions directory {}",
            sessions_dir.display()
        )
    })?;

    let file_name = format!("{session_id}.jsonl");
    let path = sessions_dir.join(file_name);

    let file = fs::OpenOptions::new()
        .create_new(create_new)
        .append(true)
        .open(&path)
        .await
        .with_context(|| format!("failed to open session log {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    for item in events {
        let mut json =
            serde_json::to_string_pretty(&item).context("failed to serialize session event")?;
        json.push('\n');

        writer
            .write_all(json.as_bytes())
            .await
            .with_context(|| format!("failed to write session log {}", path.display()))?;
    }

    writer
        .flush()
        .await
        .with_context(|| format!("failed to flush session log {}", path.display()))?;

    Ok(())
}
