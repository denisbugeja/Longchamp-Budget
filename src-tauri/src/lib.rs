// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod helper;
mod repository;

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Message from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn update_db_path(path: &str) {
    repository::update_db_file_path(path);
}

// Section part

#[tauri::command]
fn section_list_load() -> String {
    helper::vec_to_json(repository::section_list())
}

#[tauri::command]
fn insert_new_section(title: &str, color: &str) {
    repository::insert_new_section(title, color);
}

#[tauri::command]
fn delete_section(uid: &str) {
    repository::delete_section(uid);
}

#[tauri::command]
fn update_section(uid: &str, title: &str, color: &str) {
    repository::update_section(uid, title, color);
}

// Expense part

#[tauri::command]
fn expense_list_load() -> String {
    helper::vec_to_json(repository::expense_list())
}

#[tauri::command]
fn insert_new_expense(
    title: &str,
    description: &str,
    rate: &str,
    unit_price: &str,
    section_list: &str,
) {
    let vec_section_list: Vec<&str> = helper::json_to_vec(section_list);
    repository::insert_new_expense(title, description, rate, unit_price, vec_section_list);
}

#[tauri::command]
fn update_expense(uid: &str, title: &str, description: &str, rate: &str, unit_price: &str) {
    repository::update_expense(uid, title, description, rate, unit_price);
}

#[tauri::command]
fn update_expense_section_association(uid: &str, section_list: &str) {
    let vec_section_list: Vec<&str> = helper::json_to_vec(section_list);
    repository::update_expense_section_association(uid, vec_section_list);
}

#[tauri::command]
fn get_section_expense_from_expenses_instances() -> String {
    helper::vec_to_json(repository::get_section_expense_from_expenses_instances())
}

#[tauri::command]
fn get_section_expense_from_expenses_instances_section(section_uid: &str) -> String {
    helper::vec_to_json(
        repository::get_section_expense_from_expenses_instances_section(section_uid),
    )
}

#[tauri::command]
fn get_section_expense() -> String {
    helper::vec_to_json(repository::get_section_expense())
}

#[tauri::command]
fn delete_expense(uid: &str) {
    repository::delete_expense(uid);
}

#[tauri::command]
fn update_members_count(uid: &str, members_count: &str) {
    repository::update_members_count(uid, members_count);
}

#[tauri::command]
fn add_expense_instance(section_uid: &str, expense_id: &str) {
    repository::add_expense_instance(section_uid, expense_id);
}

#[tauri::command]
fn get_calculated_expenses(section_uid: &str) -> String {
    helper::vec_to_json(repository::get_calculated_expenses(section_uid))
}

#[tauri::command]
fn get_members_count(section_uid: &str) -> i32 {
    repository::get_members_count(section_uid)
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
            expense_list_load,
            insert_new_expense,
            get_section_expense_from_expenses_instances,
            get_section_expense,
            update_expense_section_association,
            update_expense,
            delete_expense,
            update_members_count,
            add_expense_instance,
            get_calculated_expenses,
            get_section_expense_from_expenses_instances_section,
            get_members_count
        ])
        .run(tauri::generate_context!())
        .expect("error) while running tauri application");
}
