use lazy_static::lazy_static;
use rusqlite::{Connection, Result};
use serde::Serialize;
use serde_json;
use std::sync::RwLock;

mod repository;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Section {
    uid: String,
    title: String,
    color: String,
}

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
    let conn = Connection::open(String::from(file_path.clone())).expect("Unable to open file");
    execute_migrations(conn);
    println!("new File: {}", &file_path);
}

#[tauri::command()]
fn section_list_load() -> String {
    let conn: Connection =
        get_connection().expect("Impossible to load connection for unit list load");

    let mut stmt = conn
        .prepare("SELECT uid, title, color FROM sections")
        .expect("Cannot prepare query");
    let section_iter = stmt
        .query_map([], |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .expect("Cannot query sections");

    let mut section_list: Vec<Section> = vec![];
    for section in section_iter {
        section_list.push(section.expect("Cannot get section"));
    }

    let json_result = serde_json::to_string(&section_list).expect("Cannot serialize section list");
    json_result
}

fn get_connection() -> Result<Connection, rusqlite::Error> {
    let file_path = GLOBAL_FILE_PATH
        .read()
        .expect("Impossible to read file path variable");
    Connection::open(String::from(file_path.clone()))
}

fn execute_migrations(conn: Connection) {
    let arr_sql: Vec<&str> = vec![
        "CREATE TABLE IF NOT EXISTS \"sections\" (
	\"uid\"	TEXT NOT NULL UNIQUE,
	\"title\"	TEXT NOT NULL,
	\"color\"	TEXT,
	\"position\"	INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(\"uid\")
);",
        "CREATE TABLE IF NOT EXISTS \"expenses\" (
	\"uid\"	TEXT NOT NULL UNIQUE,
	\"title\"	TEXT NOT NULL,
	\"description\"	TEXT,
	\"rate\"	NUMERIC NOT NULL DEFAULT 100,
	\"unit_price\"	NUMERIC NOT NULL DEFAULT 0,
	\"position\"	INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(\"uid\")
);",
        "CREATE TABLE IF NOT EXISTS \"expense_section\" (
	\"uid_expense\"	TEXT NOT NULL,
	\"uid_section\"	TEXT NOT NULL,
	FOREIGN KEY(\"uid_expense\") REFERENCES \"expenses\"(\"uid\"),
	FOREIGN KEY(\"uid_section\") REFERENCES \"sections\"(\"uid\"),
	UNIQUE(\"uid_expense\",\"uid_section\")
);",
        "CREATE TABLE IF NOT EXISTS \"expenses_instances\" (
	\"uid\"	TEXT NOT NULL UNIQUE,
	\"expense_uid\"	TEXT NOT NULL,
	\"section_uid\"	TEXT NOT NULL,
	\"units\"	INTEGER,
	\"unit_price\"	NUMERIC,
	\"rate\"	NUMERIC,
	FOREIGN KEY(\"expense_uid\") REFERENCES \"expenses\"(\"uid\"),
	FOREIGN KEY(\"expense_uid\") REFERENCES \"sections\"(\"uid\"),
	PRIMARY KEY(\"uid\")
);",
        "
INSERT INTO sections (uid, title, color, position)
SELECT 'group','Groupe','#403f6f',0
WHERE NOT EXISTS(SELECT uid, title, color, position FROM sections WHERE uid = 'group');",
    ];

    for sql in arr_sql {
        conn.execute(sql, []).expect("Cannot execute sql");
    }
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
