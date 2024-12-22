// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod helper;
mod repository;

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Message from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command()]
fn update_db_path(path: &str) {
    repository::update_db_file_path(path);
}

#[tauri::command()]
fn section_list_load() -> String {
    helper::section_list_to_json()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            update_db_path,
            section_list_load
        ])
        .run(tauri::generate_context!())
        .expect("error) while running tauri application");
}
