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
    helper::vec_to_json(helper::section_list())
}

#[tauri::command()]
fn expense_list_load() -> String {
    helper::vec_to_json(helper::expense_list())
}

#[tauri::command()]
fn insert_new_section(title: &str, color: &str) {
    repository::insert_new_section(title, color);
}

#[tauri::command()]
fn insert_new_expense(title: &str, description: &str, rate: &str, unit_price: &str) {
    repository::insert_new_expense(title, description, rate, unit_price);
}

#[tauri::command()]
fn delete_section(uid: &str) {
    if !is_allowed_to_delete_section(uid) {
        return;
    }
    repository::delete_section(uid);
}

#[tauri::command()]
fn update_section(uid: &str, title: &str, color: &str) {
    repository::update_section(uid, title, color);
}

#[tauri::command()]
fn is_allowed_to_delete_section(uid: &str) -> bool {
    uid != "group"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            update_db_path,
            section_list_load,
            insert_new_section,
            delete_section,
            update_section,
            is_allowed_to_delete_section,
            expense_list_load,
            insert_new_expense,
        ])
        .run(tauri::generate_context!())
        .expect("error) while running tauri application");
}
