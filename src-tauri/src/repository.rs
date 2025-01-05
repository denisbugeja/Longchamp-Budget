use crate::helper::{Expense, Section, SectionExpense};
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
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "INSERT INTO sections (uid, title, color, position) VALUES (?1, ?2, ?3, 0)",
        params!(Uuid::new_v4().to_string(), title, color),
        &conn,
    );
}

pub fn section_list() -> Vec<Section> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT uid, title, color FROM sections",
        [],
        |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
            })
        },
        &conn,
    )
}

pub fn expense_list() -> Vec<Expense> {
    let conn = get_connection().expect("Cannot get connection");
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
        &conn,
    )
}

pub fn delete_section(uid: &str) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql("DELETE FROM sections WHERE uid = ?1", params!(uid), &conn);
}

pub fn update_section(uid: &str, title: &str, color: &str) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "UPDATE sections SET title = ?1, color = ?2 WHERE uid = ?3",
        params!(title, color, uid),
        &conn,
    );
}

pub fn insert_new_expense(
    title: &str,
    description: &str,
    rate: &str,
    unitprice: &str,
    section_list: Vec<&str>,
) {
    let mut conn = get_connection().expect("Cannot get connection");
    let sections_in_db = section_list_from_uid_vec(section_list, &conn);
    if sections_in_db.len() == 0 {
        return;
    }

    let rate_f32: f32 = rate.parse().expect("Failed to parse rate as f32");
    let unitprice_f32: f32 = unitprice.parse().expect("Failed to parse unitprice as f32");
    let uid_expense = Uuid::new_v4().to_string();

    let tx = conn
        .transaction()
        .expect("Impossible to create transaction");

    tx.execute(
        "INSERT INTO expenses (uid, title, description, rate, unit_price, position) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        params!(uid_expense, title, description, rate_f32, unitprice_f32),
    ).expect("Failed to add query to transaction");

    for section in sections_in_db {
        tx.execute(
            "INSERT INTO expense_section (uid_expense, uid_section) VALUES (?1, ?2)",
            params!(uid_expense, section.uid),
        )
        .expect("Failed to add query to transaction");
    }

    let _ = tx.commit().expect("Failed to commit transaction");
}

fn section_list_from_uid_vec(section_list: Vec<&str>, conn: &Connection) -> Vec<Section> {
    let mut section_list_vec: Vec<Section> = vec![];
    for section in section_list {
        let mut sections_in_db = execute_read_sql(
            "SELECT uid, title, color FROM sections WHERE uid = ?1",
            params!(section),
            |row| {
                Ok(Section {
                    uid: row.get(0)?,
                    title: row.get(1)?,
                    color: row.get(2)?,
                })
            },
            conn,
        );
        if sections_in_db.len() > 0 {
            section_list_vec.push(sections_in_db.pop().expect("Impossible to pop section"));
        }
    }
    section_list_vec
}

pub fn update_expense(
    uid: &str,
    title: &str,
    description: &str,
    rate: &str,
    unitprice: &str
) {
    let conn = get_connection().expect("Cannot get connection");
    
    let rate_f32: f32 = rate.parse().expect("Failed to parse rate as f32");
    let unitprice_f32: f32 = unitprice
        .parse()
        .expect("Failed to parse unit_price as f32");

    execute_write_sql(
        "UPDATE expenses set title = ?1, description = ?2, rate = ?3, unit_price = ?4, position = ?5 WHERE uid = ?6",
        params!(title, description, rate_f32, unitprice_f32, 0, uid), 
        &conn
    );
}


pub fn update_expense_section_association(uid_expense: &str, section_list: Vec<&str>) {
    let mut conn = get_connection().expect("Cannot get connection");
    let sections_used_as_instances: Vec<SectionExpense> =
        get_section_expense_from_instances(uid_expense, &conn);
    let sections_in_db: Vec<Section> = section_list_from_uid_vec(section_list, &conn);
    if sections_in_db.len() == 0 {
        return;
    }

    let sections_used: Vec<&str> = sections_used_as_instances
        .iter()
        .map(|s: &SectionExpense| -> &str { s.uid_section.as_str() })
        .collect();

    let sections_in_update: Vec<&str> = sections_in_db
        .iter()
        .map(|s: &Section| -> &str { s.uid.as_str() })
        .collect();

    let diff: Vec<&str> = sections_used
        .iter()
        .filter(|x| !sections_in_update.contains(x))
        .cloned()
        .collect();

    if diff.len() != 0 {
        return;
    }

    let tx = conn.transaction().expect("Impossible to create transaction");

    tx.execute(
        "DELETE FROM expense_section WHERE uid_expense = ?1",
        params!(uid_expense),
    )
    .expect("Failed to add query to transaction");

    for section in sections_in_db {
        tx.execute(
            "INSERT INTO expense_section (uid_expense, uid_section) VALUES (?1, ?2)",
            params!(uid_expense, section.uid),
        )
        .expect("Failed to add query to transaction");
    }

    let _ = tx.commit().expect("Failed to commit transaction");
}


pub fn delete_expense(uid: &str) {
    let mut conn = get_connection().expect("Cannot get connection");
    let tx = conn
        .transaction()
        .expect("Impossible to create transaction");

    tx.execute(
        "DELETE FROM expense_section WHERE uid_expense = ?1",
        params!(uid),
    )
    .expect("Failed to add query to transaction");

    tx.execute("DELETE FROM expenses WHERE uid = ?1", params!(uid))
        .expect("Failed to add query to transaction");

    let _ = tx.commit().expect("Failed to commit transaction");
}

pub fn execute_write_sql<T: rusqlite::Params>(sql: &str, params: T, conn: &Connection) {
    let mut statement = conn.prepare_cached(sql).expect("Cannot prepare statement");
    statement.execute(params).expect("Cannot execute write sql");
}

pub fn is_expense_used(uid: &str) -> bool {
    let conn = get_connection().expect("Cannot get connection");
    let count: i32 = execute_read_sql(
        "SELECT COUNT(uid) FROM expenses_instances WHERE uid_expense = ?1",
        params!(uid),
        |row| Ok(row.get(0)?),
        &conn,
    )
    .pop()
    .expect("Cannot get count");
    count > 0
}

pub fn get_section_expense() -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT expense_section.uid_section, expense_section.uid_expense, sections.title AS title_section, expenses.title AS title_expense
        FROM expense_section
        INNER JOIN sections ON expense_section.uid_section = sections.uid
        INNER JOIN expenses ON expense_section.uid_expense = expenses.uid",
        [],
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
            })
        },
        &conn
    )
}

pub fn get_section_expense_from_instances(uid: &str, conn: &Connection) -> Vec<SectionExpense> {
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense
        FROM expenses_instances
        INNER JOIN sections ON expenses_instances.uid_section = sections.uid
        INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
        WHERE expenses.uid = ?1",
        params!(uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
            })
        },
        &conn
    )
}

pub fn get_section_expense_from_expenses_instances() -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
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
        &conn
    )
}

pub fn execute_read_sql<F, T, P: rusqlite::Params>(
    sql: &str,
    params: P,
    row_closure: F,
    conn: &Connection,
) -> Vec<T>
where
    F: FnMut(&Row) -> Result<T, rusqlite::Error>,
{
    let data_iter: Vec<T> = conn
        .prepare_cached(sql)
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
