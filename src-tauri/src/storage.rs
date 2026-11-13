use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn get_storage_path() -> PathBuf {
    if let Ok(app_data) = std::env::var("APPDATA") {
        let p = PathBuf::from(app_data).join("Raser").join("storage.json");
        return p;
    }
    PathBuf::from("storage.json")
}

pub fn read_storage_raw() -> String {
    let path = get_storage_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() && (trimmed.starts_with('{') || trimmed.starts_with('[')) {
                return content;
            }
        }
    }
    "{}".to_string()
}

pub fn read_storage_map() -> HashMap<String, serde_json::Value> {
    let raw = read_storage_raw();
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_storage_map(data: HashMap<String, serde_json::Value>) {
    let path = get_storage_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json_str) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(&path, json_str);
    }
}
