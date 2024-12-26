use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result, Row};
use std::sync::RwLock;
use uuid::Uuid;

lazy_static! {
    static ref GLOBAL_FILE_PATH: RwLock<String> = RwLock::new(String::from(""));
}

pub fn get_connection() -> Result<Connection, rusqlite::Error> {
    let file_path = GLOBAL_FILE_PATH
        .read()
        .expect("Impossible to read file path variable");
    Connection::open(String::from(file_path.clone()))
}

pub fn update_db_file_path(path: &str) {
    let mut file_path = GLOBAL_FILE_PATH.write().unwrap();
    *file_path = String::from(path);
    let conn = Connection::open(String::from(file_path.clone())).expect("Unable to open file");
    execute_migrations(conn);
}

pub fn insert_new_section(title: &str, color: &str) {
    get_connection()
        .expect("Impossible to load connection")
        .execute(
            "INSERT INTO sections (uid, title, color, position) VALUES (?1, ?2, ?3, 0)",
            (Uuid::new_v4().to_string(), title, color),
        )
        .expect("Cannot insert new section");
}

pub fn delete_section(uid: &str) {
    get_connection()
        .expect("Impossible to load connection")
        .execute("DELETE FROM sections WHERE uid = ?1", params!(uid))
        .expect("Cannot delete section");
}

pub fn update_section(uid: &str, title: &str, color: &str) {
    get_connection()
        .expect("Impossible to load connection")
        .execute(
            "UPDATE sections SET title = ?1, color = ?2 WHERE uid = ?3",
            params!(title, color, uid),
        )
        .expect("Cannot update section");
}

pub fn execute_read_sql<F, T>(sql: &str, row_closure: F) -> Vec<T>
where
    F: FnMut(&Row) -> Result<T, rusqlite::Error>,
{
    let data_iter: Vec<T> = self::get_connection()
        .expect("Impossible to load connection")
        .prepare(sql)
        .expect("Cannot prepare query")
        .query_map([], row_closure)
        .expect("Cannot execute query_map")
        .into_iter()
        .flatten()
        .collect();
    data_iter
}

pub fn execute_migrations(conn: Connection) {
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
