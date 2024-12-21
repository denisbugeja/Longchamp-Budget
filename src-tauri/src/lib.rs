use lazy_static::lazy_static;
use rusqlite::{Connection, Result};
use std::sync::RwLock;

lazy_static! {
    static ref GLOBAL_FILE_PATH: RwLock<String> = RwLock::new(String::from(""));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Message from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command()]
fn update_db_path(path: &str) {
    let mut file_path = GLOBAL_FILE_PATH.write().unwrap();
    *file_path = String::from(path);
    Connection::open(String::from(file_path.clone())).expect("Unable to open file");

    println!("new File: {}", &file_path);
}

fn get_connection() -> Result<Connection, rusqlite::Error> {
    let file_path = GLOBAL_FILE_PATH.read().unwrap();
    Connection::open(String::from(file_path.clone()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, update_db_path])
        .run(tauri::generate_context!())
        .expect("error) while running tauri application");
}
