use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    pub uid: String,
    pub title: String,
    pub color: String,
    pub members_count: i32,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionExpense {
    pub uid_section: String,
    pub uid_expense: String,
    pub title_section: String,
    pub title_expense: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatedExpense {
    pub uid_expense_instance: String,
    pub uid_section: String,
    pub uid_expense: String,
    pub title_section: String,
    pub title_expense: String,
    pub comments: String,
    pub section_color: String,
    pub expenses_units: i32,
    pub expenses_unit_price: f32,
    pub expenses_rate: f32,
    pub expenses_instances_units: i32,
    pub expenses_instances_unit_price: f32,
    pub expenses_instances_rate: f32,
    pub live_units: f32,
    pub live_unit_price: f32,
    pub live_rate: f32,
    pub group_rate: f32,
    pub applyed_price: f32,
    pub total_applyed_price: f32,
    pub total_inital_price: f32,
    pub group_applyed_total_price: f32,
}

pub fn vec_to_json<T: Serialize>(vec_data: Vec<T>) -> String {
    serde_json::to_string(&vec_data).expect("Cannot serialize section list")
}

pub fn json_to_vec(json_data: &str) -> Vec<&str> {
    serde_json::from_str(json_data).expect("Cannot deserialize section list")
}
