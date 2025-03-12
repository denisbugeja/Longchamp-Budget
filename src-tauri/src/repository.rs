use crate::helper::{CalculatedExpense, Expense, Section, SectionExpense, SumExpenseInstance};
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
        "SELECT uid, title, color, members_count FROM sections",
        [],
        |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
                members_count: row.get(3)?,
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
    let mut conn = get_connection().expect("Cannot get connection");

    let count: i32 = execute_read_sql(
        "SELECT COUNT(uid) FROM expenses_instances WHERE uid_section = ?1",
        params!(uid),
        |row| Ok(row.get(0)?),
        &conn,
    )
    .pop()
    .expect("Cannot get count");
    if count > 0 {
        return;
    }

    let tx = conn.transaction().expect("Impossible to create transaction");

    tx.execute(
        "DELETE FROM expense_section WHERE uid_section = ?1",
        params!(uid),
    )
    .expect("Failed to add query to transaction");

    tx.execute(
        "DELETE FROM sections WHERE uid = ?1",
        params!(uid),
    )
    .expect("Failed to add query to transaction");

    let _ = tx.commit().expect("Failed to commit transaction");
}

pub fn update_section(uid: &str, title: &str, color: &str) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "UPDATE sections SET title = ?1, color = ?2 WHERE uid = ?3",
        params!(title, color, uid),
        &conn,
    );
}

pub fn update_members_count(uid: &str, members_count: &str) {
    let conn = get_connection().expect("Cannot get connection");
    let members_count_i32: i32 = members_count.parse().expect("Failed to parse rate as i32");
    execute_write_sql(
        "UPDATE sections SET members_count = ?1 WHERE uid = ?2",
        params!(members_count_i32, uid),
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
            "SELECT uid, title, color, members_count FROM sections WHERE uid = ?1",
            params!(section),
            |row| {
                Ok(Section {
                    uid: row.get(0)?,
                    title: row.get(1)?,
                    color: row.get(2)?,
                    members_count: row.get(3)?,
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

fn parse_f_or_none(s: &str) -> Option<f32> {
    s.trim().parse().ok()
}

fn parse_i_or_none(s: &str) -> Option<i32> {
    if let Ok(value) = s.trim().parse::<i32>() {
        return Some(value);
    }

    match parse_f_or_none(s) {
        Some(value) => Some(value.floor() as i32),
        None => None,
    }
}

pub fn update_expense_instance(uid_expense_instance: &str, unit_price: &str, units: &str, rate: &str) {
    let conn = get_connection().expect("Cannot get connection");

    let unit_price_f32 = parse_f_or_none(unit_price);
    let units_i32 = parse_i_or_none(units).and_then(|value| if 0 == value { None } else { Some(value) });
    let rate_f32 = parse_f_or_none(rate);

    execute_write_sql(
        "UPDATE expenses_instances SET units = ?1, unit_price = ?2, rate = ?3 WHERE uid = ?4",
        params!(units_i32, unit_price_f32, rate_f32, uid_expense_instance), 
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

    let count: i32 = execute_read_sql(
        "SELECT COUNT(uid) FROM expenses_instances WHERE uid_expense = ?1",
        params!(uid),
        |row| Ok(row.get(0)?),
        &conn,
    )
    .pop()
    .expect("Cannot get count");
    if count > 0 {
        return;
    }

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
                count: 0
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
                count: 0
            })
        },
        &conn
    )
}

pub fn get_section_expense_from_expenses_instances() -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense, COUNT(uid_expense) AS cnt_uid_expense
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
                count: row.get(4)?
            })
        },
        &conn
    )
}

pub fn get_members_count(section_uid: &str) ->i32 {
    let conn = get_connection().expect("Cannot get connection");
    let member_count_list: Vec<i32> = execute_read_sql(
        "SELECT members_count FROM sections WHERE uid = ?1",
        params!(section_uid),
        |row| {
            Ok(row.get(0)?)
        },
        &conn
    );
    if 0 != member_count_list.len() {
        return member_count_list[0];
    }
    0
}

pub fn get_section_expense_from_expenses_instances_section(section_uid: &str) -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql("SELECT expense_section.uid_section, expense_section.uid_expense, sections.title AS title_section, expenses.title AS title_expense, count(expenses_instances.uid_expense) AS cnt_uid_expense
    FROM expense_section
    INNER JOIN sections ON expense_section.uid_section = sections.uid
    INNER JOIN expenses ON expense_section.uid_expense = expenses.uid
    LEFT JOIN expenses_instances ON 
        expense_section.uid_section = expenses_instances.uid_section 
        AND expense_section.uid_expense = expenses_instances.uid_expense
    WHERE expense_section.uid_section = ?1
    GROUP BY expenses_instances.uid_section, expenses_instances.uid_expense",
        params!(section_uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: row.get(4)?
            })
        },
        &conn
    )
}

pub fn get_calculated_expenses(section_uid: &str)-> Vec<CalculatedExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql("SELECT uid_expense_instance, uid_section, uid_expense, title_section, title_expense, comments, section_color, expenses_units,
expenses_unit_price, expenses_rate, expenses_instances_units, expenses_instances_unit_price, expenses_instances_rate,
live_units, live_unit_price, live_rate, group_rate, applyed_price, total_applyed_price, total_inital_price, group_applyed_total_price, group_applyed_unit_price
FROM view_calculated_expenses_sections_instances
WHERE uid_section = ?1
",
        params!(section_uid),
        |row| {
            Ok(CalculatedExpense {
                uid_expense_instance: row.get(0)?,
                uid_section: row.get(1)?,
                uid_expense: row.get(2)?,
                title_section: row.get(3)?,
                title_expense: row.get(4)?,
                comments: row.get(5)?,
                section_color: row.get(6)?,
                expenses_units: row.get(7)?,
                expenses_unit_price: row.get(8)?,
                expenses_rate: row.get(9)?,
                expenses_instances_units: row.get(10)?,
                expenses_instances_unit_price: row.get(11)?,
                expenses_instances_rate: row.get(12)?,
                live_units: row.get(13)?,
                live_unit_price: row.get(14)?,
                live_rate: row.get(15)?,
                group_rate: row.get(16)?,
                applyed_price: row.get(17)?,
                total_applyed_price: row.get(18)?,
                total_inital_price: row.get(19)?,
                group_applyed_total_price: row.get(20)?,
                group_applyed_unit_price: row.get(21)?,
            })
        },
        &conn
    )
}

pub fn get_sum_calculated_expenses(section_uid: &str) -> SumExpenseInstance {
    let conn = get_connection().expect("Cannot get connection");
    let results : Vec <SumExpenseInstance> = execute_read_sql("SELECT SUM(applyed_price) AS applyed_price, SUM(total_applyed_price) AS total_applyed_price
    FROM view_calculated_expenses_sections_instances
    WHERE uid_section = ?1", params!(section_uid), |row| {
        Ok(SumExpenseInstance{
            sum_unit: row.get(0)?,
            sum_total: row.get(1)?,
        })
    }, &conn);
    sum_expense_instance_from_vec(results)
}

pub fn get_group_sum_calculated_expenses() -> SumExpenseInstance {
    let conn = get_connection().expect("Cannot get connection");
    let results : Vec <SumExpenseInstance> = execute_read_sql("SELECT SUM(group_applyed_unit_price) AS sum_group_applyed_unit_price, SUM(group_applyed_total_price) AS sum_group_applyed_total_price
    FROM view_calculated_expenses_sections_instances
    WHERE group_rate <> 0", [], |row| {
        Ok(SumExpenseInstance{
            sum_unit: row.get(0)?,
            sum_total: row.get(1)?,
        })
    }, &conn);
    sum_expense_instance_from_vec(results)
}

fn sum_expense_instance_from_vec(vec : Vec<SumExpenseInstance>) -> SumExpenseInstance {
    if let Some(item) = vec.into_iter().next() {
        item
    } else {
        SumExpenseInstance{sum_unit: 0 as f32, sum_total: 0 as f32}
    }
}

pub fn get_group_calculated_expenses() -> Vec<CalculatedExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql("SELECT uid_expense_instance, uid_section, uid_expense, title_section, title_expense, comments, section_color, expenses_units,
    expenses_unit_price, expenses_rate, expenses_instances_units, expenses_instances_unit_price, expenses_instances_rate,
    live_units, live_unit_price, live_rate, group_rate, applyed_price, total_applyed_price, total_inital_price, group_applyed_total_price, group_applyed_unit_price
    FROM view_calculated_expenses_sections_instances
    WHERE group_rate <> 0",
    [],
    |row| {
            Ok(CalculatedExpense {
                uid_expense_instance: row.get(0)?,
                uid_section: row.get(1)?,
                uid_expense: row.get(2)?,
                title_section: row.get(3)?,
                title_expense: row.get(4)?,
                comments: row.get(5)?,
                section_color: row.get(6)?,
                expenses_units: row.get(7)?,
                expenses_unit_price: row.get(8)?,
                expenses_rate: row.get(9)?,
                expenses_instances_units: row.get(10)?,
                expenses_instances_unit_price: row.get(11)?,
                expenses_instances_rate: row.get(12)?,
                live_units: row.get(13)?,
                live_unit_price: row.get(14)?,
                live_rate: row.get(15)?,
                group_rate: row.get(16)?,
                applyed_price: row.get(17)?,
                total_applyed_price: row.get(18)?,
                total_inital_price: row.get(19)?,
                group_applyed_total_price: row.get(20)?,
                group_applyed_unit_price: row.get(21)?,
            })
        },
        &conn)
}

pub fn add_expense_instance(section_uid: &str, expense_id: &str) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "INSERT INTO expenses_instances (uid, uid_section, uid_expense) VALUES (?1, ?2, ?3)",
        params!(Uuid::new_v4().to_string(), section_uid, expense_id),
        &conn,
    );
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
    \"members_count\" INTEGER NOT NULL DEFAULT 0,
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
    \"comments\" TEXT,
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
"DROP VIEW IF EXISTS \"view_expenses_sections_instances\";",
"CREATE VIEW \"view_expenses_sections_instances\" AS
SELECT 
expenses_instances.uid AS uid_expense_instance,
expenses_instances.uid_section, 
expenses_instances.uid_expense, 
sections.title AS title_section, 
expenses.title AS title_expense,
expenses_instances.comments AS comments,
sections.color AS section_color,

sections.members_count AS expenses_units,
expenses.unit_price AS expenses_unit_price,
expenses.rate AS expenses_rate,

expenses_instances.units AS expenses_instances_units,
expenses_instances.unit_price AS expenses_instances_unit_price,
expenses_instances.rate AS expenses_instances_rate,

CASE WHEN expenses_instances.units IS NOT NULL AND TRIM(expenses_instances.units,\" \") != \"\"
    THEN expenses_instances.units
    ELSE sections.members_count
END AS live_units,

CASE WHEN expenses_instances.unit_price IS NOT NULL AND TRIM(expenses_instances.unit_price ,\" \") != \"\"
    THEN CAST(expenses_instances.unit_price AS REAL)
    ELSE CAST(expenses.unit_price AS REAL)
END AS live_unit_price,

CASE WHEN expenses_instances.rate  IS NOT NULL AND TRIM(expenses_instances.rate,\" \") != \"\"
    THEN CAST(expenses_instances.rate AS REAL)
    ELSE CAST(expenses.rate AS REAL)
END AS live_rate,
CASE WHEN group_sections.members_count > 0 
    THEN group_sections.members_count
    ELSE 1
END AS group_members_count
FROM sections AS group_sections, expenses_instances
INNER JOIN sections ON expenses_instances.uid_section = sections.uid
INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
WHERE group_sections.uid = 'group'",
"DROP VIEW IF EXISTS \"view_calculated_expenses_sections_instances\";",
"CREATE VIEW \"view_calculated_expenses_sections_instances\" AS
SELECT uid_expense_instance, uid_section, uid_expense, title_section, title_expense, comments, section_color, expenses_units,
expenses_unit_price, expenses_rate, expenses_instances_units, expenses_instances_unit_price, expenses_instances_rate,
live_units, live_unit_price, live_rate,
(100 - view_expenses_sections_instances.live_rate) AS group_rate,
ROUND(view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100), 2) AS applyed_price,
ROUND(view_expenses_sections_instances.live_units * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100), 2) AS total_applyed_price,
ROUND(view_expenses_sections_instances.live_units * view_expenses_sections_instances.live_unit_price, 2) AS total_inital_price,
ROUND(view_expenses_sections_instances.live_units * view_expenses_sections_instances.live_unit_price - view_expenses_sections_instances.live_units * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100),2) AS group_applyed_total_price,
ROUND(((view_expenses_sections_instances.live_units * view_expenses_sections_instances.live_unit_price - view_expenses_sections_instances.live_units * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100)) / group_members_count), 2) AS group_applyed_unit_price
FROM view_expenses_sections_instances",
"CREATE TRIGGER IF NOT EXISTS update_group_members_count_after_update
AFTER UPDATE OF members_count ON sections
FOR EACH ROW
BEGIN
    UPDATE sections
    SET members_count = (SELECT SUM(members_count) FROM sections WHERE uid != 'group')
    WHERE uid = 'group';
END;",
"CREATE TRIGGER IF NOT EXISTS update_group_members_count_after_insert
AFTER INSERT ON sections
FOR EACH ROW
BEGIN
    UPDATE sections
    SET members_count = (SELECT SUM(members_count) FROM sections WHERE uid != 'group')
    WHERE uid = 'group';
END;",
"CREATE TRIGGER IF NOT EXISTS update_group_members_count_after_delete
AFTER DELETE ON sections
FOR EACH ROW
BEGIN
    UPDATE sections
    SET members_count = (SELECT SUM(members_count) FROM sections WHERE uid != 'group')
    WHERE uid = 'group';
END;",
    ];

    for sql in arr_sql {
        conn.execute(sql, []).expect("Cannot execute sql");
    }
}
