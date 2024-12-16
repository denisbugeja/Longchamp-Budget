use lazy_static::lazy_static;
use std::sync::RwLock;
use tauri::menu::{MenuBuilder, MenuEvent, MenuItemBuilder};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

lazy_static! {
    static ref GLOBAL_FILE_PATH: RwLock<String> = RwLock::new(String::from(""));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Message from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn change_file_path(app_handle: &AppHandle) {
    let mut file_path = GLOBAL_FILE_PATH.write().unwrap();

    let selected_file_path = app_handle.dialog().file().blocking_pick_file();
    match selected_file_path {
        Some(path) => {
            *file_path = String::from(path.as_path().unwrap().to_str().unwrap());
        }
        None => {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app).unwrap();
            let hide = MenuItemBuilder::new("Hide").id("hide").build(app).unwrap();
            let show = MenuItemBuilder::new("Show").id("show").build(app).unwrap();

            // we could opt handle an error case better than calling unwrap
            let menu = MenuBuilder::new(app)
                .items(&[&quit, &hide, &show])
                .build()
                .unwrap();
            app.set_menu(menu).expect("Failed to set menu");
            Ok(())
        })
        .on_menu_event(|_app, event: MenuEvent| {
            change_file_path(_app.app_handle());
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error) while running tauri application");
}
