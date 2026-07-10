use crate::parser::AgentEventItem;

use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

// static APP_PATH: &str = ".mimi";

fn get_app_dir() -> Result<PathBuf, String> {
    let app_dir;

    if let Some(dir) = std::env::var_os("HOME") {
        app_dir = PathBuf::from(dir).join(".mimi");
    } else {
        return Err("Could not find the home dir".to_string());
    }

    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    Ok(app_dir)
}

pub async fn append_events(
    session_id: &String,
    events: &Vec<AgentEventItem>,
    create_new: bool,
) -> Result<(), String> {
    if session_id.is_empty() {
        return Err("session Id is required".to_string());
    }

    let sessions_dir = get_app_dir().map_err(|e| e.to_string())?.join("sessions");

    fs::create_dir_all(&sessions_dir).map_err(|e| e.to_string())?;

    let file_name = format!("{session_id}.jsonl");
    let path = sessions_dir.join(file_name);

    let file = OpenOptions::new()
        .create_new(create_new)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    let mut writer = BufWriter::new(file);

    for item in events {
        let json = serde_json::to_string_pretty(&item).map_err(|e| e.to_string())?;
        writeln!(writer, "{json}").map_err(|e| e.to_string())?;
    }

    writer.flush().map_err(|e| e.to_string())?;

    Ok(())
}
