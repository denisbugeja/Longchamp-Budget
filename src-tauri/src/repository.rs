use crate::helper::{CalculatedExpense, Expense, Fq, FqSection, Section, SectionExpense, SumExpenseInstance, FqTotal};
use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result, Row};
use std::sync::RwLock;
use uuid::Uuid;
use std::fs;
use std::path::Path;

lazy_static! {
    static ref GLOBAL_FILE_PATH: RwLock<String> = RwLock::new(String::from(""));
}

pub fn get_global_file_path() -> String {
    let path = GLOBAL_FILE_PATH.read().expect("Impossible to read file path variable");
    path.clone()
}

fn parse_s_or_none(s: &str) -> Option<String> {
    s.trim().parse().ok()
}

fn parse_f_or_none(s: &str) -> Option<f32> {
    s.trim().parse().ok()
}

#[allow(dead_code)]
fn parse_i_or_none(s: &str) -> Option<i32> {
    if let Ok(value) = s.trim().parse::<i32>() {
        return Some(value);
    }

    parse_f_or_none(s).map(|value| value.floor() as i32)
}

pub fn get_connection() -> Result<Connection, rusqlite::Error> {
    let file_path = GLOBAL_FILE_PATH
        .read()
        .expect("Impossible to read file path variable");
    Connection::open(file_path.clone())
}

pub fn update_db_file_path(str_path: &str, erase_if_exists: bool) {
    let mut file_path = GLOBAL_FILE_PATH.write().expect("Impossible to get file path for write");
    let mut real_path = String::from(str_path);
    if !real_path.ends_with(".lb") {
        real_path.push_str(".lb");
    }
    let path = Path::new(&real_path);
    
    if erase_if_exists && path.exists() {
        fs::remove_file(path).expect("Impossible to erase file")
    }

    *file_path = real_path.clone();
    let conn = Connection::open(real_path).expect("Unable to open file");

    execute_migrations(conn);
}

pub fn insert_new_section(title: &str, color: &str, members_count: i32, adults_count: i32) {
    let conn = get_connection().expect("Cannot get connection");

    let existing_sections: Vec<Section> = execute_read_sql(
        "SELECT uid, title, color, members_count, adults_count FROM sections WHERE title = ?1",
        params!(title),
        |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
                members_count: row.get(3)?,
                adults_count: row.get(4)?,
            })
        },
        &conn,
    );

    if !existing_sections.is_empty(){
        return;
    }

    execute_write_sql(
        "INSERT INTO sections (uid, title, color, members_count, adults_count, position) VALUES (?1, ?2, ?3, ?4, ?5, (SELECT COALESCE(MAX(position), -1) + 1 FROM sections))",
        params!(Uuid::new_v4().to_string(), title, color, members_count.abs(), adults_count.abs()),
        &conn,
    );
}


pub fn insert_new_fq(title: &str, coeff: &str, national_contribution: &str, online_commission_rate: &str, online_commission_fees: &str) {
    let conn = get_connection().expect("Cannot get connection");

    let existing_fqs: Vec<Fq> = execute_read_sql(
        "SELECT uid, title, coeff, national_contribution, online_commission_rate, online_commission_fees FROM fqs WHERE title = ?1",
        params!(title),
        |row| {
            Ok(Fq {
                uid: row.get(0)?,
                title: row.get(1)?,
                coeff: row.get(2)?,
                national_contribution: row.get(3)?,
                online_commission_rate: row.get(4)?,
                online_commission_fees: row.get(5)?,
            })
        },
        &conn, 
    );

    if !existing_fqs.is_empty(){
        return;
    }

    let coeff_f32: f32 = coeff.parse().expect("Failed to parse coeff as f32");
    let national_contribution_f32: f32 = national_contribution.parse().expect("Failed to parse national_contribution as f32");
    let online_commission_rate_f32: f32 = online_commission_rate.parse().expect("Failed to parse online_commission_rate as f32");
    let online_commission_fees_f32: f32 = online_commission_fees.parse().expect("Failed to parse online_commission_fees as f32");

    execute_write_sql(
        "INSERT INTO fqs (uid, title, coeff, national_contribution, online_commission_rate, online_commission_fees, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, (SELECT COALESCE(MAX(position), -1) + 1 FROM fqs))",
        params!(Uuid::new_v4().to_string(), title, coeff_f32, national_contribution_f32, online_commission_rate_f32, online_commission_fees_f32),
        &conn,
    );
}


pub fn section_list() -> Vec<Section> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT uid, title, color, members_count, adults_count FROM sections ORDER BY position ASC",
        [],
        |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
                members_count: row.get(3)?,
                adults_count: row.get(4)?,
            })
        },
        &conn,
    )
}

pub fn fq_list() -> Vec<Fq> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT uid, title, coeff, national_contribution, online_commission_rate, online_commission_fees FROM fqs ORDER BY position ASC",
        [],
        |row| {
            Ok(Fq {
                uid: row.get(0)?,
                title: row.get(1)?,
                coeff: row.get(2)?,
                national_contribution: row.get(3)?,
                online_commission_rate: row.get(4)?,
                online_commission_fees: row.get(5)?
            })
        },
        &conn,
    )
}

pub fn fq_section_list_load(section_uid: &str) -> Vec<FqSection> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT sections_fqs.uid_section, sections_fqs.uid_fq, sections_fqs.members_count, sections.title as section_title, fqs.title as fq_title
        FROM sections_fqs INNER JOIN sections ON sections_fqs.uid_section = sections.uid 
        INNER JOIN fqs ON fqs.uid = sections_fqs.uid_fq
        WHERE sections_fqs.uid_section = ?1 
        ORDER BY fqs.position ASC",
        params!(section_uid),
        |row| {
            Ok(FqSection {
                uid_fq: row.get(1)?,
                uid_section: row.get(0)?,
                members_count: row.get(2)?,
                title_section: row.get(3)?,
                title_fq: row.get(4)?,
            })
        },
        &conn,
    )
}

pub fn get_fqs_calculated_by_section(section_uid: &str) -> Vec<FqTotal> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT title_section, title_fq, uid_fq, uid_section, declared_unit_price, declared_group_unit_price, coeff, calculated_unit_price_with_coeff, group_calculated_unit_price, total_group_member_price, national_contribution, total_member_price, national_commission, total FROM view_calculated_fqs_total WHERE uid_section = ?",
        params!(section_uid),
        |row| {
            Ok(FqTotal {
                title_section: row.get(0)?,
                title_fq: row.get(1)?,
                uid_fq: row.get(2)?,
                uid_section: row.get(3)?,
                declared_unit_price: row.get(4)?,
                declared_group_unit_price: row.get(5)?,
                coeff: row.get(6)?,
                calculated_unit_price_with_coeff: row.get(7)?,
                group_calculated_unit_price: row.get(8)?,
                total_group_member_price: row.get(9)?,
                national_contribution: row.get(10)?,
                total_member_price: row.get(11)?,
                national_commission: row.get(12)?,
                total: row.get(13)?,
            })
        },
        &conn,
    )
}

pub fn expense_list() -> Vec<Expense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT uid, title, description, rate, unit_price, position  FROM expenses ORDER BY position ASC",
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
        |row| row.get(0),
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

    tx.commit().expect("Failed to commit transaction");
}

pub fn delete_fq(uid: &str) {
    let mut conn = get_connection().expect("Cannot get connection");
    let tx = conn.transaction().expect("Impossible to create transaction");

    tx.execute(
        "DELETE FROM fqs WHERE uid = ?1",
        params!(uid),
    )
    .expect("Failed to add query to transaction");

    tx.commit().expect("Failed to commit transaction");
}

pub fn update_section(uid: &str, title: &str, color: &str, members_count: i32, adults_count: i32) {
    let conn = get_connection().expect("Cannot get connection");

    let existing_sections: Vec<Section> = execute_read_sql(
        "SELECT uid, title, color, members_count, adults_count FROM sections WHERE title = ?1 AND uid != ?2",
        params!(title, uid),
        |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
                members_count: row.get(3)?,
                adults_count: row.get(4)?,
            })
        },
        &conn,
    );

    if !existing_sections.is_empty(){
        return;
    }

    execute_write_sql(
        "UPDATE sections SET title = ?1, color = ?2, members_count=?3, adults_count=?4 WHERE uid = ?5",
        params!(title, color, members_count.abs(), adults_count.abs(), uid),
        &conn,
    );
}


pub fn update_fq(uid: &str, title: &str, coeff: &str, national_contribution: &str, online_commission_rate: &str, online_commission_fees: &str) {
    let conn = get_connection().expect("Cannot get connection");

    let existing_fqs: Vec<Fq> = execute_read_sql(
        "SELECT uid, title, coeff, national_contribution, online_commission_rate, online_commission_fees FROM fqs WHERE title = ?1 and uid != ?2",
        params!(title, uid),
        |row| {
            Ok(Fq {
                uid: row.get(0)?,
                title: row.get(1)?,
                coeff: row.get(2)?,
                national_contribution: row.get(3)?,
                online_commission_rate: row.get(4)?,
                online_commission_fees: row.get(5)?,
            })
        },
        &conn,
    );

    if !existing_fqs.is_empty(){
        return;
    }

    let coeff_f32: f32 = coeff.parse().expect("Failed to parse coeff as f32");
    let national_contribution_f32: f32 = national_contribution.parse().expect("Failed to parse national_contribution as f32");
    let online_commission_rate_f32: f32 = online_commission_rate.parse().expect("Failed to parse online_commission_rate as f32");
    let online_commission_fees_f32: f32 = online_commission_fees.parse().expect("Failed to parse online_commission_fees as f32");

    execute_write_sql(
        "UPDATE fqs SET title = ?1, coeff = ?2, national_contribution=?3, online_commission_rate=?4, online_commission_fees=?5 WHERE uid = ?6",
        params!(title, coeff_f32, national_contribution_f32, online_commission_rate_f32, online_commission_fees_f32, uid),
        &conn,
    );
}

pub fn update_section_order(section_list: Vec<&str>)
{
    let mut conn = get_connection().expect("Cannot get connection");
    let tx = conn.transaction().expect("Impossible to create transaction");

    for (index, uid) in section_list.iter().enumerate() {
        tx.execute(
            "UPDATE sections SET position = ?1 WHERE uid = ?2",
            params!(index, uid),
        )
        .expect("Failed to add query to transaction");
    }

    tx.commit().expect("Failed to commit transaction");
}

pub fn update_fq_order(fq_list: Vec<&str>)
{
    let mut conn = get_connection().expect("Cannot get connection");
    let tx = conn.transaction().expect("Impossible to create transaction");

    for (index, uid) in fq_list.iter().enumerate() {
        tx.execute(
            "UPDATE fqs SET position = ?1 WHERE uid = ?2",
            params!(index, uid),
        )
        .expect("Failed to add query to transaction");
    }

    tx.commit().expect("Failed to commit transaction");
}

pub fn update_members_count(uid: &str, members_count: i32) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "UPDATE sections SET members_count = ?1 WHERE uid = ?2",
        params!(members_count.abs(), uid),
        &conn,
    );
}

pub fn update_adults_count(uid: &str, adults_count: i32) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "UPDATE sections SET adults_count = ?1 WHERE uid = ?2",
        params!(adults_count.abs(), uid),
        &conn,
    );
}

pub fn update_fq_section_members_count(section_uid: &str, fq_uid: &str, members_count: i32) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "UPDATE sections_fqs SET members_count = ?1 WHERE uid_section = ?2 AND uid_fq = ?3",
        params!(members_count.abs(), section_uid, fq_uid),
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
    if sections_in_db.is_empty() {
        return;
    }

    let rate_f32: f32 = rate.parse().expect("Failed to parse rate as f32");
    let unitprice_f32: f32 = unitprice.parse().expect("Failed to parse unitprice as f32");
    let uid_expense = Uuid::new_v4().to_string();

    let tx = conn
        .transaction()
        .expect("Impossible to create transaction");

    tx.execute(
        "INSERT INTO expenses (uid, title, description, rate, unit_price, position) VALUES (?1, ?2, ?3, ?4, ?5, (SELECT COALESCE(MAX(position), -1) + 1 FROM expenses))",
        params!(uid_expense, title, description, rate_f32, unitprice_f32),
    ).expect("Failed to add query to transaction");

    for section in sections_in_db {
        tx.execute(
            "INSERT INTO expense_section (uid_expense, uid_section) VALUES (?1, ?2)",
            params!(uid_expense, section.uid),
        )
        .expect("Failed to add query to transaction");
    }

    tx.commit().expect("Failed to commit transaction");
}

fn section_list_from_uid_vec(section_list: Vec<&str>, conn: &Connection) -> Vec<Section> {
    let mut section_list_vec: Vec<Section> = vec![];
    for section in section_list {
        let mut sections_in_db = execute_read_sql(
            "SELECT uid, title, color, members_count, adults_count FROM sections WHERE uid = ?1",
            params!(section),
            |row| {
                Ok(Section {
                    uid: row.get(0)?,
                    title: row.get(1)?,
                    color: row.get(2)?,
                    members_count: row.get(3)?,
                    adults_count: row.get(4)?,
                })
            },
            conn,
        );
        if !sections_in_db.is_empty() {
            section_list_vec.push(sections_in_db.pop().expect("Impossible to pop section"));
        }
    }
    section_list_vec
}


pub fn get_members_fq_count_by_section(section_uid: &str) -> i32{
    let conn = get_connection().expect("Cannot get connection");
    let count: i32 = execute_read_sql(
        "SELECT COALESCE(SUM(members_count),0) AS total FROM sections_fqs WHERE uid_section = ?1",
        params!(section_uid),
        |row| row.get(0),
        &conn,
    )
    .pop()
    .expect("Cannot get count");
    count
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
        "UPDATE expenses SET title = ?1, description = ?2, rate = ?3, unit_price = ?4 WHERE uid = ?5",
        params!(title, description, rate_f32, unitprice_f32, uid), 
        &conn
    );
}

pub fn update_expense_instance_order(vec_expense_instance_list: Vec<&str>)
{
    let mut conn = get_connection().expect("Cannot get connection");
    let tx = conn.transaction().expect("Impossible to create transaction");

    for (index, uid) in vec_expense_instance_list.iter().enumerate() {
        tx.execute(
            "UPDATE expenses_instances SET position = ?1 WHERE uid = ?2",
            params!(index, uid),
        )
        .expect("Failed to add query to transaction");
    }

    tx.commit().expect("Failed to commit transaction");
}

pub fn update_expense_instance(uid_expense_instance: &str, unit_price: &str, number: &str, units: &str, units_adults: &str, rate: &str, comments: &str) {
    let conn = get_connection().expect("Cannot get connection");

    let unit_price_f32 = parse_f_or_none(unit_price);
    let units_f32 = parse_f_or_none(units);
    let units_adults_f32 = parse_f_or_none(units_adults);
    let rate_f32 = parse_f_or_none(rate);
    let comments_s = parse_s_or_none(comments);
    let number_f32 = parse_f_or_none(number);

    if number_f32.is_none() {
        return;
    }

    execute_write_sql(
        "UPDATE expenses_instances SET units = ?1, units_adults=?2, unit_price = ?3, rate = ?4, comments=?5, number = ?6 WHERE uid = ?7",
        params!(units_f32, units_adults_f32, unit_price_f32, rate_f32, comments_s, number_f32, uid_expense_instance),
        &conn
    );
}

pub fn delete_expense_instance(uid_expense_instance: &str) {
    let conn = get_connection().expect("Cannot get connection");
    
    execute_write_sql(
        "DELETE FROM expenses_instances WHERE uid = ?1",
        params!(uid_expense_instance), 
        &conn
    );
}

pub fn copy_expense_instance(uid_expense_instance: &str) {
    let conn = get_connection().expect("Cannot get connection");

    execute_write_sql(
        "INSERT INTO  expenses_instances (uid, uid_expense, uid_section, comments, number, units, units_adults, unit_price, rate, position) SELECT ?1 AS uid, uid_expense, uid_section, comments, number, units, units_adults, unit_price, rate, (SELECT COALESCE(MAX(position), -1) + 1 FROM expenses_instances) AS position FROM expenses_instances WHERE uid = ?2",
        params!(Uuid::new_v4().to_string(), uid_expense_instance), 
        &conn
    );
}

pub fn update_expense_section_association(uid_expense: &str, section_list: Vec<&str>) {
    let mut conn = get_connection().expect("Cannot get connection");
    let sections_used_as_instances: Vec<SectionExpense> =
        get_section_expense_from_instances(uid_expense, &conn);
    let sections_in_db: Vec<Section> = section_list_from_uid_vec(section_list, &conn);
    if sections_in_db.is_empty() {
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

    if !diff.is_empty() {
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

    tx.commit().expect("Failed to commit transaction");
}


pub fn delete_expense(uid: &str) {
    let mut conn = get_connection().expect("Cannot get connection");

    let count: i32 = execute_read_sql(
        "SELECT COUNT(uid) FROM expenses_instances WHERE uid_expense = ?1",
        params!(uid),
        |row| row.get(0),
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

    tx.commit().expect("Failed to commit transaction");
}

pub fn execute_write_sql<T: rusqlite::Params>(sql: &str, params: T, conn: &Connection) {
    let mut statement = conn.prepare_cached(sql).expect("Cannot prepare statement");
    statement.execute(params).expect("Cannot execute write sql");
}

pub fn get_section_expense() -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT expense_section.uid_section, expense_section.uid_expense, sections.title AS title_section, expenses.title AS title_expense, expenses.description
        FROM expense_section
        INNER JOIN sections ON expense_section.uid_section = sections.uid
        INNER JOIN expenses ON expense_section.uid_expense = expenses.uid
        ORDER BY sections.position ASC, expenses.position ASC",
        [],
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: 0,
                description: row.get(4)?
            })
        },
        &conn
    )
}

pub fn get_section_expense_cnt_from_instance(section_uid: &str, expense_uid: &str) -> f32 {
    let conn = get_connection().expect("Cannot get connection");
    let result = execute_read_sql(
        "SELECT ROUND(SUM(number),2) AS cnt
        FROM expenses_instances
        WHERE uid_section = ?1
        AND uid_expense = ?2
        GROUP BY expenses_instances.uid_expense",
        params!(section_uid, expense_uid),
        |row| {
            row.get(0)
        },
        &conn
    );
    let mut count: f32 = 0.0;
    if !result.is_empty() {
        count = result[0];
    }
    count
}

pub fn get_section_expense_from_instance(section_uid: &str, expense_uid: &str) -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense, expenses.description
        FROM expenses_instances
        INNER JOIN sections ON expenses_instances.uid_section = sections.uid
        INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
        WHERE expenses_instances.uid_section = ?1 
        AND expenses_instances.uid_expense = ?2
        ORDER BY expenses_instances.position ASC",
        params!(section_uid, expense_uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: 0,
                description: row.get(4)?,
            })
        },
        &conn
    )
}

pub fn get_section_expense_from_association(section_uid: &str, expense_uid: &str) -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT DISTINCT expense_section.uid_section, expense_section.uid_expense, sections.title AS title_section, expenses.title AS title_expense, expenses.description
        FROM expense_section
        INNER JOIN sections ON expense_section.uid_section = sections.uid
        INNER JOIN expenses ON expense_section.uid_expense = expenses.uid
        WHERE expense_section.uid_section = ?1 
        AND expense_section.uid_expense = ?2
        ORDER BY sections.position ASC",
        params!(section_uid, expense_uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: 0,
                description: row.get(4)?,
            })
        },
        &conn
    )
}


pub fn get_section_expense_from_instances_wrapper(expense_uid: &str)-> Vec<SectionExpense>
{
    let conn = get_connection().expect("Cannot get connection");
    get_section_expense_from_instances(expense_uid, &conn)
}

fn get_section_expense_from_instances(expense_uid: &str, conn: &Connection) -> Vec<SectionExpense> {
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense, expenses.description
        FROM expenses_instances
        INNER JOIN sections ON expenses_instances.uid_section = sections.uid
        INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
        WHERE expenses.uid = ?1
        ORDER BY expenses_instances.position ASC",
        params!(expense_uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: 0,
                description: row.get(4)?
            })
        },
        conn
    )
}

pub fn get_section_expense_from_expenses_instances() -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense, COUNT(uid_expense) AS cnt_uid_expense, expenses.description
        FROM expenses_instances
        INNER JOIN sections ON expenses_instances.uid_section = sections.uid
        INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
        GROUP BY expenses_instances.uid_expense
        ORDER BY expenses_instances.position ASC",
        [],
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: row.get(4)?,
                description: row.get(5)?
            })
        },
        &conn
    )
}

pub fn get_section_expense_from_expenses_instances_and_section(section_uid: &str) -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql(
        "SELECT expenses_instances.uid_section, expenses_instances.uid_expense, sections.title AS title_section, expenses.title AS title_expense, COUNT(uid_expense) AS cnt_uid_expense, expenses.description
        FROM expenses_instances
        INNER JOIN sections ON expenses_instances.uid_section = sections.uid
        INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
        WHERE expenses_instances.uid_section = ?1
        GROUP BY expenses_instances.uid_expense
        ORDER BY expenses.position ASC
        ",
        params!(section_uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: row.get(4)?,
                description: row.get(5)?
            })
        },
        &conn
    )
}

pub fn get_members_count(section_uid: &str) ->i32 {
    let conn = get_connection().expect("Cannot get connection");
    let members_count_list: Vec<i32> = execute_read_sql(
        "SELECT members_count FROM sections WHERE uid = ?1",
        params!(section_uid),
        |row| {
            row.get(0)
        },
        &conn
    );
    if !members_count_list.is_empty() {
        return members_count_list[0];
    }
    0
}

pub fn get_adults_count(section_uid: &str) ->i32 {
    let conn = get_connection().expect("Cannot get connection");
    let adults_count_list: Vec<i32> = execute_read_sql(
        "SELECT adults_count FROM sections WHERE uid = ?1",
        params!(section_uid),
        |row| {
            row.get(0)
        },
        &conn
    );
    if !adults_count_list.is_empty() {
        return adults_count_list[0];
    }
    0
}

pub fn get_section_expense_from_expenses_instances_section(section_uid: &str) -> Vec<SectionExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql("SELECT expense_section.uid_section, expense_section.uid_expense, sections.title AS title_section, expenses.title AS title_expense, expenses.description
    FROM expense_section
    INNER JOIN sections ON expense_section.uid_section = sections.uid
    INNER JOIN expenses ON expense_section.uid_expense = expenses.uid
    WHERE expense_section.uid_section = ?1 
    GROUP BY sections.uid, expenses.uid
    ORDER BY expenses.position ASC",
        params!(section_uid),
        |row| {
            Ok(SectionExpense {
                uid_section: row.get(0)?,
                uid_expense: row.get(1)?,
                title_section: row.get(2)?,
                title_expense: row.get(3)?,
                count: 0,
                description: row.get(4)?
            })
        },
        &conn
    )
}

pub fn get_calculated_expenses(section_uid: &str)-> Vec<CalculatedExpense> {
    let conn = get_connection().expect("Cannot get connection");
    execute_read_sql("SELECT uid_expense_instance, uid_section, uid_expense, title_section, title_expense, comments, section_color, expenses_units, expenses_units_adults,
expenses_unit_price, expenses_rate, expenses_instances_units, expenses_instances_units_adults, expenses_instances_unit_price, expenses_instances_rate,
live_units, live_units_adults, live_unit_price, live_rate, group_rate, applyed_price, total_applyed_price, total_inital_price, group_applyed_total_price, group_applyed_unit_price, group_members_count, expenses_description, expenses_instances_number
FROM view_calculated_expenses_sections_instances
WHERE uid_section = ?1
ORDER BY expenses_instances_position ASC",
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
                expenses_units_adults: row.get(8)?,
                expenses_unit_price: row.get(9)?,
                expenses_rate: row.get(10)?,
                expenses_instances_units: row.get(11)?,
                expenses_instances_units_adults: row.get(12)?,
                expenses_instances_unit_price: row.get(13)?,
                expenses_instances_rate: row.get(14)?,
                live_units: row.get(15)?,
                live_units_adults: row.get(16)?,
                live_unit_price: row.get(17)?,
                live_rate: row.get(18)?,
                group_rate: row.get(19)?,
                applyed_price: row.get(20)?,
                total_applyed_price: row.get(21)?,
                total_inital_price: row.get(22)?,
                group_applyed_total_price: row.get(23)?,
                group_applyed_unit_price: row.get(24)?,
                group_members_count: row.get(25)?,
                expenses_description: row.get(26)?,
                expenses_instances_number: row.get(27)?
            })
        },
        &conn
    )
}

pub fn get_total_per_member(section_uid: &str) -> SumExpenseInstance {
    let conn = get_connection().expect("Cannot get connection");
    let results : Vec <SumExpenseInstance> = execute_read_sql("SELECT ROUND(SUM(sum_group_applyed_unit_price),2) AS sum_group_applyed_unit_price, ROUND(SUM(sum_group_applyed_total_price),2) AS sum_group_applyed_total_price FROM 
(
    SELECT SUM(total_applyed_price / expenses_units) AS sum_group_applyed_unit_price, SUM(total_applyed_price) AS sum_group_applyed_total_price
    FROM view_calculated_expenses_sections_instances
    WHERE uid_section = ?1
UNION ALL    
    SELECT SUM(group_applyed_unit_price) AS sum_group_applyed_unit_price, SUM(group_applyed_total_price) AS sum_group_applyed_total_price
    FROM view_calculated_expenses_sections_instances
    WHERE group_rate <> 0
UNION ALL
    SELECT SUM(total_applyed_price / group_members_count) AS sum_group_applyed_unit_price, SUM(total_applyed_price) AS sum_group_applyed_total_price
    FROM view_calculated_expenses_sections_instances
    WHERE uid_section = 'group'
)", params!(section_uid), |row| {
        Ok(SumExpenseInstance{
            sum_unit: row.get(0)?,
            sum_total: row.get(1)?,
        })
    }, &conn);
    sum_expense_instance_from_vec(results)
}


pub fn get_sum_calculated_expenses(section_uid: &str) -> SumExpenseInstance {
    let conn = get_connection().expect("Cannot get connection");
    let results : Vec <SumExpenseInstance> = execute_read_sql("SELECT ROUND(SUM(total_applyed_price / expenses_units),2) AS applyed_price, ROUND(SUM(total_applyed_price),2) AS total_applyed_price
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
    let results : Vec <SumExpenseInstance> = execute_read_sql("SELECT ROUND(SUM(sum_group_applyed_unit_price),2) AS sum_group_applyed_unit_price, ROUND(SUM(sum_group_applyed_total_price),2) AS sum_group_applyed_total_price FROM 
(
    SELECT SUM(group_applyed_unit_price) AS sum_group_applyed_unit_price, SUM(group_applyed_total_price) AS sum_group_applyed_total_price
    FROM view_calculated_expenses_sections_instances
    WHERE group_rate <> 0
UNION ALL
    SELECT SUM(total_applyed_price / group_members_count) AS sum_group_applyed_unit_price, SUM(total_applyed_price) AS sum_group_applyed_total_price
    FROM view_calculated_expenses_sections_instances
    WHERE uid_section = 'group'
)", [], |row| {
        Ok(SumExpenseInstance{
            sum_unit: row.get(0)?,
            sum_total: row.get(1)?,
        })
    }, &conn);
    sum_expense_instance_from_vec(results)
}

pub fn get_group_only_sum_calculated_expenses() -> SumExpenseInstance {
    let conn = get_connection().expect("Cannot get connection");
    let results : Vec <SumExpenseInstance> = execute_read_sql("SELECT ROUND(SUM(group_applyed_unit_price),2) AS sum_group_applyed_unit_price, ROUND(SUM(group_applyed_total_price),2) AS sum_group_applyed_total_price
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
    execute_read_sql("SELECT uid_expense_instance, uid_section, uid_expense, title_section, title_expense, comments, section_color, expenses_units, expenses_units_adults,
    expenses_unit_price, expenses_rate, expenses_instances_units, expenses_instances_units_adults, expenses_instances_unit_price, expenses_instances_rate,
    live_units, live_units_adults, live_unit_price, live_rate, group_rate, applyed_price, total_applyed_price, total_inital_price, group_applyed_total_price, group_applyed_unit_price, group_members_count, expenses_description, expenses_instances_number
    FROM view_calculated_expenses_sections_instances
    WHERE group_rate <> 0
    ORDER BY sections_position ASC, expenses_instances_position ASC",
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
                expenses_units_adults: row.get(8)?,
                expenses_unit_price: row.get(9)?,
                expenses_rate: row.get(10)?,
                expenses_instances_units: row.get(11)?,
                expenses_instances_units_adults: row.get(12)?,
                expenses_instances_unit_price: row.get(13)?,
                expenses_instances_rate: row.get(14)?,
                live_units: row.get(15)?,
                live_units_adults: row.get(16)?,
                live_unit_price: row.get(17)?,
                live_rate: row.get(18)?,
                group_rate: row.get(19)?,
                applyed_price: row.get(20)?,
                total_applyed_price: row.get(21)?,
                total_inital_price: row.get(22)?,
                group_applyed_total_price: row.get(23)?,
                group_applyed_unit_price: row.get(24)?,
                group_members_count: row.get(25)?,
                expenses_description: row.get(26)?,
                expenses_instances_number: row.get(27)?
            })
        },
        &conn)
}

pub fn add_expense_instance(section_uid: &str, expense_id: &str) {
    let conn = get_connection().expect("Cannot get connection");
    execute_write_sql(
        "INSERT INTO expenses_instances (uid, uid_section, uid_expense, position) VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(position), -1) + 1 FROM expenses_instances WHERE uid_section = ?2))",
        params!(Uuid::new_v4().to_string(), section_uid, expense_id),
        &conn,
    );
}

pub fn update_expense_order(expense_list: Vec<&str>)
{
    let mut conn = get_connection().expect("Cannot get connection");
    let tx = conn.transaction().expect("Impossible to create transaction");

    for (index, uid) in expense_list.iter().enumerate() {
        tx.execute(
            "UPDATE expenses SET position = ?1 WHERE uid = ?2",
            params!(index, uid),
        )
        .expect("Failed to add query to transaction");
    }

    tx.commit().expect("Failed to commit transaction");
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
    \"members_count\" NUMERIC NOT NULL DEFAULT 0,
    \"adults_count\" NUMERIC NOT NULL DEFAULT 0,
	\"position\"	INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(\"uid\"),
    UNIQUE(\"title\")
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
    \"number\" NUMERIC NOT NULL DEFAULT 1,
	\"units\" NUMERIC,
    \"units_adults\" NUMERIC,
	\"unit_price\" NUMERIC,
	\"rate\" NUMERIC,
    \"position\"	INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY(\"uid_expense\") REFERENCES \"expenses\"(\"uid\"),
	FOREIGN KEY(\"uid_section\") REFERENCES \"sections\"(\"uid\"),
	PRIMARY KEY(\"uid\")
);",
        "CREATE TABLE IF NOT EXISTS \"fqs\" (
	\"uid\"	TEXT NOT NULL UNIQUE,
	\"title\"	TEXT NOT NULL,
    \"national_contribution\" NUMERIC NOT NULL DEFAULT 0,
    \"coeff\" NUMERIC NOT NULL DEFAULT 0,
    \"online_commission_rate\" NUMERIC NOT NULL DEFAULT 0,
    \"online_commission_fees\" NUMERIC NOT NULL DEFAULT 0,
	\"position\"	INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY(\"uid\"),
    UNIQUE(\"title\")
);",
        "CREATE TABLE IF NOT EXISTS \"sections_fqs\" (
	\"uid_section\"	TEXT NOT NULL,
    \"uid_fq\"	TEXT NOT NULL,
	\"members_count\" NUMERIC NOT NULL DEFAULT 0,
	FOREIGN KEY(\"uid_section\") REFERENCES \"sections\"(\"uid\"),
    FOREIGN KEY(\"uid_fq\") REFERENCES \"fqs\"(\"uid\"),
    UNIQUE(\"uid_section\",\"uid_fq\")
);",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSE_SECTION_UID_EXPENSE\" ON \"expense_section\" (\"uid_expense\");",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSE_SECTION_UID_SECTION\" ON \"expense_section\" (\"uid_section\");",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSES_INSTANCES_UID_SECTION\" ON \"expenses_instances\" (\"uid_section\");",
        "CREATE INDEX IF NOT EXISTS \"IX_EXPENSES_INSTANCES_UID_EXPENSE\" ON \"expenses_instances\" (\"uid_expense\");",
        "CREATE INDEX IF NOT EXISTS \"IX_SECTIONS_FQS_UID_SECTION\" ON \"sections_fqs\" (\"uid_section\");",
        "CREATE INDEX IF NOT EXISTS \"IX_SECTIONS_FQS_UID_FQ\" ON \"sections_fqs\" (\"uid_fq\");",
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
sections.position AS sections_position,
expenses.title AS title_expense,
expenses.position AS expenses_position,
expenses_instances.comments AS comments,
expenses_instances.position AS expenses_instances_position,
expenses_instances.number AS expenses_instances_number,
sections.color AS section_color,

sections.members_count AS expenses_units,
sections.adults_count AS expenses_units_adults,
expenses.unit_price AS expenses_unit_price,
expenses.rate AS expenses_rate,
expenses.description AS expenses_description,

expenses_instances.units AS expenses_instances_units,
expenses_instances.units_adults AS expenses_instances_units_adults,
expenses_instances.unit_price AS expenses_instances_unit_price,
expenses_instances.rate AS expenses_instances_rate,

CASE WHEN expenses_instances.units IS NOT NULL AND TRIM(expenses_instances.units,\" \") != \"\"
    THEN expenses_instances.units
    ELSE sections.members_count
END AS live_units,

CASE WHEN expenses_instances.units_adults IS NOT NULL AND TRIM(expenses_instances.units_adults,\" \") != \"\"
    THEN expenses_instances.units_adults
    ELSE sections.adults_count
END AS live_units_adults,

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
END AS group_members_count,

CASE WHEN group_sections.adults_count > 0 
    THEN group_sections.adults_count
    ELSE 1
END AS group_adults_count

FROM sections AS group_sections, expenses_instances
INNER JOIN sections ON expenses_instances.uid_section = sections.uid
INNER JOIN expenses ON expenses_instances.uid_expense = expenses.uid
WHERE group_sections.uid = 'group'",
"DROP VIEW IF EXISTS \"view_calculated_expenses_sections_instances\";",
"CREATE VIEW \"view_calculated_expenses_sections_instances\" AS
SELECT uid_expense_instance, uid_section, uid_expense, title_section, title_expense, comments, section_color, expenses_units, expenses_units_adults,
expenses_unit_price, expenses_rate, expenses_instances_units, expenses_instances_units_adults, expenses_instances_unit_price, expenses_instances_rate,
live_units, live_unit_price, live_rate,
(100 - view_expenses_sections_instances.live_rate) AS group_rate,
ROUND(view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100), 2) AS applyed_price,
ROUND((view_expenses_sections_instances.expenses_instances_number * (view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults)) * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100), 2) AS total_applyed_price,
ROUND((view_expenses_sections_instances.expenses_instances_number * (view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults)) * view_expenses_sections_instances.live_unit_price, 2) AS total_inital_price,
ROUND((view_expenses_sections_instances.expenses_instances_number * (view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults)) * view_expenses_sections_instances.live_unit_price - (view_expenses_sections_instances.expenses_instances_number * (view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults)) * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100),2) AS group_applyed_total_price,
ROUND((((view_expenses_sections_instances.expenses_instances_number * (view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults)) * view_expenses_sections_instances.live_unit_price - (view_expenses_sections_instances.expenses_instances_number * (view_expenses_sections_instances.live_units + view_expenses_sections_instances.live_units_adults)) * view_expenses_sections_instances.live_unit_price * (view_expenses_sections_instances.live_rate / 100)) / group_members_count), 2) AS group_applyed_unit_price,
group_members_count, group_adults_count, live_units_adults, expenses_instances_position, expenses_position, sections_position, expenses_description, expenses_instances_number
FROM view_expenses_sections_instances",
"DROP TRIGGER IF EXISTS \"update_group_members_count_after_update\";",
"CREATE TRIGGER update_group_members_count_after_update
AFTER UPDATE ON sections
FOR EACH ROW
BEGIN
    UPDATE sections
    SET members_count = COALESCE((SELECT SUM(COALESCE(members_count, 0)) FROM sections WHERE uid != 'group'), 0),
        adults_count = COALESCE((SELECT SUM(COALESCE(adults_count, 0)) FROM sections WHERE uid != 'group'), 0)
    WHERE uid = 'group';
END;",
"DROP TRIGGER IF EXISTS \"update_group_members_count_after_insert\";",
"CREATE TRIGGER update_group_members_count_after_insert
AFTER INSERT ON sections
FOR EACH ROW
BEGIN
    UPDATE sections
    SET members_count = COALESCE((SELECT SUM(COALESCE(members_count, 0)) FROM sections WHERE uid != 'group'), 0),
        adults_count = COALESCE((SELECT SUM(COALESCE(adults_count, 0)) FROM sections WHERE uid != 'group'), 0)
    WHERE uid = 'group';
END;",
"DROP TRIGGER IF EXISTS \"update_group_members_count_after_delete\";",
"CREATE TRIGGER update_group_members_count_after_delete
AFTER DELETE ON sections
FOR EACH ROW
BEGIN
    UPDATE sections
    SET members_count = COALESCE((SELECT SUM(COALESCE(members_count, 0)) FROM sections WHERE uid != 'group'), 0),
        adults_count = COALESCE((SELECT SUM(COALESCE(adults_count, 0)) FROM sections WHERE uid != 'group'), 0)
    WHERE uid = 'group';
END;",
"DROP TRIGGER IF EXISTS \"insert_sections_fqs_after_insert_sections\";",
"CREATE TRIGGER insert_sections_fqs_after_insert_sections
AFTER INSERT ON sections
FOR EACH ROW
BEGIN
    INSERT INTO sections_fqs (uid_section, uid_fq, members_count) SELECT sections.uid AS uid_section, fqs.uid AS uid_fq, 0 FROM sections, fqs WHERE sections.uid = NEW.uid;
END;",
"DROP TRIGGER IF EXISTS \"insert_sections_fqs_after_insert_fqs\";",
"CREATE TRIGGER insert_sections_fqs_after_insert_fqs
AFTER INSERT ON fqs
FOR EACH ROW
BEGIN
    INSERT INTO sections_fqs (uid_section, uid_fq, members_count) SELECT sections.uid AS uid_section, fqs.uid AS uid_fq, 0 FROM sections, fqs WHERE fqs.uid = NEW.uid;
END;",
"DROP TRIGGER IF EXISTS \"delete_sections_fqs_after_delete_sections\";",
"CREATE TRIGGER delete_sections_fqs_after_delete_sections
BEFORE DELETE ON sections
FOR EACH ROW
BEGIN
    DELETE FROM sections_fqs WHERE uid_section = OLD.uid;
    UPDATE sections_fqs 
    SET members_count = (
        SELECT SUM(COALESCE(members_count, 0))
        FROM sections_fqs s2 
        WHERE s2.uid_section != 'group' 
        AND s2.uid_fq = sections_fqs.uid_fq
    )
    WHERE uid_section = 'group';
END;",
"DROP TRIGGER IF EXISTS \"update_sections_fqs_after_update_sections_fqs\";",
"CREATE TRIGGER update_sections_fqs_after_update_sections_fqs
AFTER UPDATE ON sections_fqs
FOR EACH ROW
BEGIN
    UPDATE sections_fqs 
    SET members_count = (
        SELECT SUM(COALESCE(members_count, 0))
        FROM sections_fqs s2 
        WHERE s2.uid_section != 'group' 
        AND s2.uid_fq = sections_fqs.uid_fq
    )
    WHERE uid_section = 'group';
END;",
"DROP TRIGGER IF EXISTS \"delete_sections_fqs_after_delete_fqs\";",
"CREATE TRIGGER delete_sections_fqs_after_delete_fqs
BEFORE DELETE ON fqs
FOR EACH ROW
BEGIN
    DELETE FROM sections_fqs WHERE uid_fq = OLD.uid;
    UPDATE sections_fqs 
    SET members_count = (
        SELECT SUM(COALESCE(members_count, 0))
        FROM sections_fqs s2 
        WHERE s2.uid_section != 'group' 
        AND s2.uid_fq = sections_fqs.uid_fq
    )
    WHERE uid_section = 'group';
END;",
"DROP VIEW IF EXISTS \"view_declared_sections_fq_members\";",
"CREATE VIEW \"view_declared_sections_fq_members\" AS
SELECT uid_section, COALESCE(SUM(members_count),0) AS total_members_fq_declared
FROM sections_fqs
GROUP BY uid_section",
"DROP VIEW IF EXISTS \"view_declared_fqs_sections_total_price\";",
"CREATE VIEW \"view_declared_fqs_sections_total_price\" AS
SELECT view_declared_sections_fq_members.uid_section, 
ROUND(SUM(total_applyed_price / expenses_units * view_declared_sections_fq_members.total_members_fq_declared),2) AS total_declared
FROM view_calculated_expenses_sections_instances INNER JOIN view_declared_sections_fq_members ON view_calculated_expenses_sections_instances.uid_section = view_declared_sections_fq_members.uid_section
GROUP BY view_declared_sections_fq_members.uid_section",
"DROP VIEW IF EXISTS \"view_declared_fqs_sections_total_members\";",
"CREATE VIEW \"view_declared_fqs_sections_total_members\" AS
SELECT sections_fqs.uid_section, ROUND(COALESCE(SUM(fqs.coeff * sections_fqs.members_count),0),2) as fqs_total_members FROM sections_fqs
INNER JOIN fqs ON sections_fqs.uid_fq = fqs.uid
GROUP BY sections_fqs.uid_section",
"DROP VIEW IF EXISTS \"view_declared_fqs_group_unit_price\";",
//TODO FIX THIS VIEW TO GET EXACT DATA - PONDERATED GROUP UNIT VALUE IS NOT CALCULATED - THIS IS ONLY GROUP UNIT PRICE
"CREATE VIEW \"view_declared_fqs_group_unit_price\" AS
SELECT 'group' AS uid_section, ROUND(SUM(sum_group_applyed_unit_price),2) AS declared_unit_price
FROM 
(
    SELECT SUM(group_applyed_unit_price) AS sum_group_applyed_unit_price
    FROM view_calculated_expenses_sections_instances
    WHERE group_rate <> 0
UNION ALL
    SELECT SUM(total_applyed_price / group_members_count) AS sum_group_applyed_unit_price
    FROM view_calculated_expenses_sections_instances
    WHERE uid_section = 'group'
)",
"DROP VIEW IF EXISTS \"view_declared_fqs_sections_unit_price\";",
"CREATE VIEW \"view_declared_fqs_sections_unit_price\" AS
SELECT view_declared_fqs_sections_total_price.uid_section, 
ROUND(view_declared_fqs_sections_total_price.total_declared / view_declared_fqs_sections_total_members.fqs_total_members,2) AS declared_unit_price
FROM view_declared_fqs_sections_total_price INNER JOIN view_declared_fqs_sections_total_members ON view_declared_fqs_sections_total_price.uid_section  = view_declared_fqs_sections_total_members.uid_section
WHERE view_declared_fqs_sections_total_price.uid_section <> 'group'
UNION ALL
SELECT view_declared_fqs_group_unit_price.uid_section, 
ROUND(view_declared_fqs_group_unit_price.declared_unit_price * view_declared_sections_fq_members.total_members_fq_declared / view_declared_fqs_sections_total_members.fqs_total_members, 2) AS declared_unit_price
FROM view_declared_fqs_group_unit_price 
INNER JOIN view_declared_fqs_sections_total_members ON view_declared_fqs_group_unit_price.uid_section = view_declared_fqs_sections_total_members.uid_section
INNER JOIN view_declared_sections_fq_members ON view_declared_fqs_group_unit_price.uid_section = view_declared_sections_fq_members.uid_section",
"DROP VIEW IF EXISTS \"view_declared_calculated_fqs_sections_unit_price\";",
"CREATE VIEW \"view_declared_calculated_fqs_sections_unit_price\" AS
SELECT sections_fqs.uid_fq, sections_fqs.uid_section, view_declared_fqs_sections_unit_price.declared_unit_price, 
ROUND(fqs.coeff,2) as coeff, 
ROUND(view_declared_fqs_sections_unit_price.declared_unit_price * fqs.coeff,2) AS calculated_unit_price_with_coeff
FROM view_declared_fqs_sections_unit_price INNER JOIN sections_fqs ON view_declared_fqs_sections_unit_price.uid_section = sections_fqs.uid_section 
INNER JOIN fqs ON sections_fqs.uid_fq = fqs.uid",
"DROP VIEW IF EXISTS \"view_calculated_fqs_total\";",
"CREATE VIEW \"view_calculated_fqs_total\" AS
SELECT sections.title as title_section, fqs.title as title_fq, s.uid_fq, s.uid_section, s.declared_unit_price, g.declared_unit_price as declared_group_unit_price, s.coeff, s.calculated_unit_price_with_coeff,
g.calculated_unit_price_with_coeff AS group_calculated_unit_price,
ROUND(s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff,2) AS total_group_member_price,
fqs.national_contribution,
ROUND(s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution ,2) AS total_member_price,
ROUND((s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution) * fqs.online_commission_rate + fqs.online_commission_fees,2) AS national_commission,
ROUND(s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution ,2) + ROUND((s.calculated_unit_price_with_coeff + g.calculated_unit_price_with_coeff + fqs.national_contribution) * fqs.online_commission_rate + fqs.online_commission_fees,2) AS total
FROM view_declared_calculated_fqs_sections_unit_price AS s INNER JOIN view_declared_calculated_fqs_sections_unit_price AS g ON s.uid_fq = g.uid_fq
INNER JOIN fqs ON s.uid_fq = fqs.uid
INNER JOIN sections ON s.uid_section  = sections.uid
AND g.uid_section = 'group'
ORDER BY sections.position, fqs.position ASC"
    ];

    for sql in arr_sql {
        conn.execute(sql, []).expect("Cannot execute sql");
    }
}
