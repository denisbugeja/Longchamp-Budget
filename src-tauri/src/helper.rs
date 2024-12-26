use crate::repository;
use serde::Serialize;
use serde_json;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Section {
    uid: String,
    title: String,
    color: String,
}

pub fn section_list() -> Vec<Section> {
    raw_section_list("SELECT uid, title, color FROM sections")
}

fn raw_section_list(sql: &str) -> Vec<Section> {
    repository::execute_read_sql(sql, |row| {
        Ok(Section {
            uid: row.get(0)?,
            title: row.get(1)?,
            color: row.get(2)?,
        })
    })
}

pub fn vec_to_json<T: Serialize>(vec_data: Vec<T>) -> String {
    serde_json::to_string(&vec_data).expect("Cannot serialize section list")
}
