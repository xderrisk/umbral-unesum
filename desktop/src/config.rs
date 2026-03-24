use directories::ProjectDirs;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub fn get_devices_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("edu", "unesum", "umbral")
        .expect("No se pudieron determinar los directorios del proyecto");
    let config_dir = proj_dirs.config_dir().to_path_buf();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Error al crear carpeta de configuración");
    }

    config_dir.join("devices.json")
}

pub fn get_app_data_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("edu", "unesum", "umbral")
        .expect("No se pudo obtener el directorio de proyecto");
    let local_dir = proj_dirs.data_local_dir().to_path_buf();
    if !local_dir.exists() {
        fs::create_dir_all(&local_dir).expect("Error al crear carpeta de configuración");
    }
    proj_dirs.data_local_dir().to_path_buf()
}

pub fn news_folder() -> PathBuf {
    let ruta = get_app_data_path();
    ruta.join("news/prueba1.png")
}
pub fn load_aulas() -> Vec<Value> {
    let ruta = get_devices_path();

    if !ruta.exists() {
        fs::write(&ruta, "[]").expect("No se pudo inicializar devices.json");
        return Vec::new();
    }

    let data = fs::read_to_string(ruta).unwrap_or_else(|_| "[]".to_string());
    let v: Value = serde_json::from_str(&data).unwrap_or(Value::Array(Vec::new()));

    v.as_array().cloned().unwrap_or_default()
}
