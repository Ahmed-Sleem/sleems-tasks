use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

struct AppState {
    data_dir: Mutex<PathBuf>,
}

#[tauri::command]
fn save_data(app: tauri::AppHandle, data: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    fs::create_dir_all(&*dir).map_err(|e| e.to_string())?;
    let path = dir.join("tasks.json");
    fs::write(path, data).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_data(app: tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();
    let dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let path = dir.join("tasks.json");
    if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

#[tauri::command]
async fn export_data(app: tauri::AppHandle, data: String, filename: String) -> Result<(), String> {
    let file_path = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name(filename)
        .blocking_save_file()
        .ok_or_else(|| "Cancelled".to_string())?;
    let path = file_path.as_path().ok_or_else(|| "Invalid path".to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_data(app: tauri::AppHandle) -> Result<String, String> {
    let file_path = app.dialog()
        .file()
        .add_filter("JSON", &["json", "txt"])
        .blocking_pick_file()
        .ok_or_else(|| "Cancelled".to_string())?;
    let path = file_path.as_path().ok_or_else(|| "Invalid path".to_string())?;
    fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data dir");
            app.manage(AppState {
                data_dir: Mutex::new(data_dir),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![save_data, load_data, export_data, import_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
