use crate::repository;
use serde::Serialize;
use serde_json;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Section {
    uid: String,
    title: String,
    color: String,
}

pub fn section_list_to_json() -> String {
    let data_list: Vec<Section> =
        repository::execute_read_sql("SELECT uid, title, color FROM sections", |row| {
            Ok(Section {
                uid: row.get(0)?,
                title: row.get(1)?,
                color: row.get(2)?,
            })
        });
    serde_json::to_string(&data_list).expect("Cannot serialize section list")
}
