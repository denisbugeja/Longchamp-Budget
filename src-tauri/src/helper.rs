use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub uid: String,
    pub title: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

pub fn json_to_vec(json_data: &str) -> Vec<&str> {
    serde_json::from_str(json_data).expect("Cannot deserialize section list")
}
