// WIP
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct Edit {
    path: String,
    old_content: String,
    new_content: String,
}

pub fn edit_file(args: Value) -> Result<Value, String> {
    let edit = serde_json::from_value::<Edit>(args).map_err(|e| e.to_string())?;

    let path = Path::new(&edit.path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success:": "ok"}))
}
