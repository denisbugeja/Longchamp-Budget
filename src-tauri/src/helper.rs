use serde::Serialize;
use serde_json;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Section {
    pub uid: String,
    pub title: String,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct Expense {
    pub uid: String,
    pub title: String,
    pub description: String,
    pub rate: f32,
    pub unit_price: f32,
    pub position: i32,
}

pub fn vec_to_json<T: Serialize>(vec_data: Vec<T>) -> String {
    serde_json::to_string(&vec_data).expect("Cannot serialize section list")
}
