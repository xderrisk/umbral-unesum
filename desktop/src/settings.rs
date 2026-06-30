use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    pub api_key: String,
    pub news: bool,
    pub unique_image: bool,
    pub unique_image_path: String,
}

pub fn load() -> Settings {
    let load_process = || -> Option<Settings> {
        let file_path = get_settings_file_path();
        if !file_path.exists() {
            return None;
        }
        let data = fs::read_to_string(file_path).ok()?;
        let settings: Settings = serde_json::from_str(&data).ok()?;
        Some(settings)
    };
    load_process().unwrap_or(Settings {
        api_key: String::new(),
        news: false,
        unique_image: false,
        unique_image_path: String::new(),
    })
}

pub fn save(settings: &Settings) -> Result<(), std::io::Error> {
    let file_path = get_settings_file_path();
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(file_path, data)?;
    Ok(())
}

pub fn get_settings_file_path() -> PathBuf {
    let proj_dirs = get_project_dirs();
    let config_dir = proj_dirs.config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create configuration directory");
    }
    let file_path = config_dir.join("settings.json");
    if !file_path.exists() {
        fs::File::create(&file_path).expect("Failed to create empty settings.json file");
    }
    file_path
}

fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("edu", "unesum", "umbral").expect("Failed to determine project directories")
}

pub fn get_devices_path() -> PathBuf {
    let proj_dirs = get_project_dirs();
    let config_dir = proj_dirs.config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create configuration directory");
    }
    config_dir.join("devices.json")
}

pub fn get_app_data_path() -> PathBuf {
    let proj_dirs = get_project_dirs();
    let local_dir = proj_dirs.data_local_dir().to_path_buf();
    if !local_dir.exists() {
        fs::create_dir_all(&local_dir).expect("Failed to create local data directory");
    }
    local_dir
}

pub fn news_folder() -> PathBuf {
    let path = get_app_data_path();
    let news_dir = path.join("news");
    if !news_dir.exists() {
        fs::create_dir_all(&news_dir).expect("Failed to create news cache directory");
    }
    news_dir
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

pub fn rename_device(mac: &str, name: &str) -> Result<(), String> {
    let path = get_devices_path();
    let mut devices = load_classrooms();
    if let Some(device) = devices.iter_mut().find(|device| {
        device
            .get("mac")
            .and_then(|m| m.as_str())
            .map(|m| m == mac)
            .unwrap_or(false)
    }) {
        device["name"] = Value::String(name.to_string());
        let json_string = serde_json::to_string_pretty(&devices)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        fs::write(path, json_string).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    } else {
        Err(format!("Device with MAC {} not found locally", mac))
    }
}

pub fn delete_device(mac: &str) -> Result<(), String> {
    let path = get_devices_path();
    let mut devices = load_classrooms();
    if let Some(index) = devices.iter().position(|device| {
        device
            .get("mac")
            .and_then(|m| m.as_str())
            .map(|m| m == mac)
            .unwrap_or(false)
    }) {
        devices.remove(index);
        let json_string = serde_json::to_string_pretty(&devices)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        std::fs::write(path, json_string).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    } else {
        Err(format!("Device with MAC {} not found locally", mac))
    }
}
