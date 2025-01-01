use crate::helper::{Expense, Section, SectionExpense};
use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result, Row};
use std::sync::RwLock;
use uuid::Uuid;

macro_rules! generate_vec_params {
    ($section_list:expr) => {
        $section_list
            .iter()
            .map(|&s| Box::new(s) as Box<dyn rusqlite::ToSql>)
            .collect::<Vec<Box<dyn rusqlite::ToSql>>>()
    };
}

macro_rules! vec_params_to_rustsqlite {
    ($params:expr) => {
        $params
            .iter()
            .map(|p| p.as_ref())
            .collect::<Vec<&dyn rusqlite::ToSql>>()
            .as_slice()
    };
}

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
    execute_write_sql(
        "INSERT INTO sections (uid, title, color, position) VALUES (?1, ?2, ?3, 0)",
        params!(Uuid::new_v4().to_string(), title, color),
    );
}

pub fn section_list() -> Vec<Section> {
    execute_read_sql("SELECT uid, title, color FROM sections", [], |row| {
        Ok(Section {
            uid: row.get(0)?,
            title: row.get(1)?,
            color: row.get(2)?,
        })
    })
}

pub fn expense_list() -> Vec<Expense> {
    execute_read_sql(
        "SELECT uid, title, description, rate, unit_price, position  FROM expenses",
        [],
        |row| {
            Ok(Expense {
                uid: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                rate: row.get(3)?,
                unit_price: row.get(4)?,
                position: row.get(5)?,
            })
        },
    )
}

pub fn delete_section(uid: &str) {
    execute_write_sql("DELETE FROM sections WHERE uid = ?1", params!(uid));
}

pub fn update_section(uid: &str, title: &str, color: &str) {
    execute_write_sql(
        "UPDATE sections SET title = ?1, color = ?2 WHERE uid = ?3",
        params!(title, color, uid),
    );
}

pub fn insert_new_expense(
    title: &str,
    description: &str,
    rate: &str,
    unitprice: &str,
    section_list: Vec<&str>,
) {
    let sections_in_db = section_list_from_uid_vec(section_list);
    if sections_in_db.len() == 0 {
        return;
    }

    let rate_f32: f32 = rate.parse().expect("Failed to parse rate as f32");
    let unitprice_f32: f32 = unitprice.parse().expect("Failed to parse unitprice as f32");
    let uid_expense = Uuid::new_v4().to_string();

    execute_write_sql(
        "INSERT INTO expenses (uid, title, description, rate, unit_price, position) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        params!(uid_expense, title, description, rate_f32, unitprice_f32),
    );

    for section in sections_in_db {
        execute_write_sql(
            "INSERT INTO expense_section (uid_expense, uid_section) VALUES (?1, ?2)",
            params!(uid_expense, section.uid),
        );
    }
}

fn section_list_from_uid_vec(section_list: Vec<&str>) -> Vec<Section> {
    let params = generate_vec_params!(section_list);
    execute_read_sql(
        "SELECT uid, title, color AS cnt FROM sections WHERE uid IN (?1)",
        vec_params_to_rustsqlite!(params),
        |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
            })
        },
    )
}

pub fn update_expense(
    uid: &str,
    title: &str,
    description: &str,
    rate: &str,
    unitprice: &str,
    section_list: Vec<&str>,
) {
    let sections_in_db = section_list_from_uid_vec(section_list);
    if sections_in_db.len() == 0 {
        return;
    }

    //TODO Now we check if we have no missing associated expenses

    //if OK

    let rate_f32: f32 = rate.parse().expect("Failed to parse rate as f32");
    let unitprice_f32: f32 = unitprice
        .parse()
        .expect("Failed to parse unit_price as f32");

    execute_write_sql(
        "UPDATE expenses set title = ?1, description = ?2, rate = ?3, unit_price = ?4, position = ?5 WHERE uid = ?6",
        params!(title, description, rate_f32, unitprice_f32, 0, uid),
    );

    execute_write_sql(
        "DELETE FROM expense_section WHERE uid_expense = ?1",
        params!(uid),
    );

    for section in sections_in_db {
        execute_write_sql(
            "INSERT INTO expense_section (uid_expense, uid_section) VALUES (?1, ?2)",
            params!(uid, section.uid),
        );
    }
}

pub fn delete_expense(uid: &str) {
    execute_write_sql(
        "DELETE FROM expense_section WHERE uid_expense = ?1",
        params!(uid),
    );
    execute_write_sql("DELETE FROM expenses WHERE uid = ?1", params!(uid));
}

pub fn execute_write_sql<T: rusqlite::Params>(sql: &str, params: T) {
    get_connection()
        .expect("Impossible to load connection")
        .execute(sql, params)
        .expect("Cannot execute write sql");
}

pub fn is_expense_used(uid: &str) -> bool {
    let count: i32 = execute_read_sql(
        "SELECT COUNT(uid) FROM expenses_instances WHERE uid_expense = ?1",
        params!(uid),
        |row| Ok(row.get(0)?),
    )
    .pop()
    .expect("Cannot get count");
    count > 0
}

pub fn get_section_expense_from_expenses_instances() -> Vec<SectionExpense> {
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense
        FROM expenses_instances
        INNER JOIN sections ON expenses_instances.uid_section = sections.uid
        INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid",
        [],
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
            })
        },
    )
}

pub fn execute_read_sql<F, T, P: rusqlite::Params>(sql: &str, params: P, row_closure: F) -> Vec<T>
where
    F: FnMut(&Row) -> Result<T, rusqlite::Error>,
{
    let data_iter: Vec<T> = self::get_connection()
        .expect("Impossible to load connection")
        .prepare(sql)
        .expect("Cannot prepare query")
        .query_map(params, row_closure)
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
	\"uid_expense\"	TEXT NOT NULL,
	\"uid_section\"	TEXT NOT NULL,
	\"units\"	INTEGER,
	\"unit_price\"	NUMERIC,
	\"rate\"	NUMERIC,
	FOREIGN KEY(\"uid_expense\") REFERENCES \"expenses\"(\"uid\"),
	FOREIGN KEY(\"uid_section\") REFERENCES \"sections\"(\"uid\"),
	PRIMARY KEY(\"uid\")
);",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSE_SECTION_UID_EXPENSE\" ON \"expense_section\" (\"uid_expense\");",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSE_SECTION_UID_SECTION\" ON \"expense_section\" (\"uid_section\");",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSES_INSTANCES_UID_SECTION\" ON \"expenses_instances\" (\"uid_section\");",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSES_INSTANCES_UID_EXPENSE\" ON \"expenses_instances\" (\"uid_expense\");",
        "INSERT INTO sections (uid, title, color, position)
SELECT 'group','Groupe','#403f6f',0
WHERE NOT EXISTS(SELECT uid, title, color, position FROM sections WHERE uid = 'group');",
    ];

    for sql in arr_sql {
        conn.execute(sql, []).expect("Cannot execute sql");
    }
}
