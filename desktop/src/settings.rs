use directories::ProjectDirs;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub fn get_devices_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("edu", "unesum", "umbral")
        .expect("Failed to determine project directories");
    let config_dir = proj_dirs.config_dir().to_path_buf();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create configuration directory");
    }

    config_dir.join("devices.json")
}

pub fn get_config_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("edu", "unesum", "umbral")
        .expect("Failed to determine project directories");
    let config_dir = proj_dirs.config_dir().to_path_buf();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create configuration directory");
    }

    config_dir.join("config.json")
}

pub fn get_app_data_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("edu", "unesum", "umbral")
        .expect("Failed to obtain project data directory");
    let local_dir = proj_dirs.data_local_dir().to_path_buf();
    if !local_dir.exists() {
        fs::create_dir_all(&local_dir).expect("Failed to create local data directory");
    }
    local_dir
}

pub fn news_folder() -> PathBuf {
    let path = get_app_data_path();
    path.join("news")
}

pub fn list_news_images() -> Vec<PathBuf> {
    let dir = news_folder();
    fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif")
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn load_classrooms() -> Vec<Value> {
    let path = get_devices_path();

    if !path.exists() {
        fs::write(&path, "[]").expect("Failed to initialize devices.json");
        return Vec::new();
    }

    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    let v: Value = serde_json::from_str(&data).unwrap_or(Value::Array(Vec::new()));

    v.as_array().cloned().unwrap_or_default()
}

pub fn save_device(uid: &str, name: &str, mac: &str) -> Result<(), String> {
    let path = get_devices_path();
    let mut devices = load_classrooms();
    let new_device = serde_json::json!({
        "uid": uid,
        "name": name,
        "mac": mac,
    });
    devices.push(new_device);
    let json_string = serde_json::to_string_pretty(&devices)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::write(path, json_string).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

pub fn save_api_key(api_key: &str) -> Result<(), String> {
    let path = get_config_path();
    let configuration = serde_json::json!({
        "api_key": api_key.trim(),
    });

    let json_string = serde_json::to_string_pretty(&configuration)
        .map_err(|e| format!("Failed to serialize configuration: {}", e))?;
    fs::write(path, json_string).map_err(|e| format!("Failed to write configuration: {}", e))?;
    Ok(())
}

pub fn load_api_key() -> Option<String> {
    let path = get_config_path();
    if !path.exists() {
        return None;
    }

    let data = fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&data).ok()?;
    v.get("api_key")
        .and_then(|k| k.as_str())
        .map(|s| s.to_string())
}
