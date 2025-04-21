use crate::repository;
use rust_xlsxwriter::{Format, FormatAlign, Formula, Workbook, Worksheet};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    pub uid: String,
    pub title: String,
    pub color: String,
    pub members_count: i32,
    pub adults_count: i32,
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
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatedExpense {
    pub uid_expense_instance: Option<String>,
    pub uid_section: Option<String>,
    pub uid_expense: Option<String>,
    pub title_section: Option<String>,
    pub title_expense: Option<String>,
    pub comments: Option<String>,
    pub section_color: Option<String>,
    pub expenses_units: Option<i32>,
    pub expenses_units_adults: Option<i32>,
    pub expenses_unit_price: Option<f32>,
    pub expenses_rate: Option<f32>,
    pub expenses_instances_units: Option<i32>,
    pub expenses_instances_units_adults: Option<i32>,
    pub expenses_instances_unit_price: Option<f32>,
    pub expenses_instances_rate: Option<f32>,
    pub live_units: Option<f32>,
    pub live_units_adults: Option<f32>,
    pub live_unit_price: Option<f32>,
    pub live_rate: Option<f32>,
    pub group_rate: Option<f32>,
    pub applyed_price: Option<f32>,
    pub total_applyed_price: Option<f32>,
    pub total_inital_price: Option<f32>,
    pub group_applyed_total_price: Option<f32>,
    pub group_applyed_unit_price: Option<f32>,
    pub group_members_count: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SumExpenseInstance {
    pub sum_unit: f32,
    pub sum_total: f32,
}

pub fn vec_to_json<T: Serialize>(vec_data: Vec<T>) -> String {
    serde_json::to_string(&vec_data).expect("Cannot serialize section list")
}

pub fn struct_to_json<T: Serialize>(struct_data: T) -> String {
    serde_json::to_string(&struct_data).expect("Cannot serialize struct list")
}

pub fn json_to_vec(json_data: &str) -> Vec<&str> {
    serde_json::from_str(json_data).expect("Cannot deserialize section list")
}

fn encode_text(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn generate_xls_file() {
    let file_path = repository::get_global_file_path();
    let path = Path::new(&file_path);
    let mut path_buf = PathBuf::from(path);

    path_buf.set_extension("xlsx");
    let final_path = path_buf.to_string_lossy().into_owned();

    let section_list: Vec<Section> = repository::section_list();
    let mut workbook = Workbook::new();

    for section in section_list {
        handle_worksheet(&section, &mut workbook);
    }
    let _ = workbook.save(final_path);
}

fn handle_worksheet(section: &Section, workbook: &mut Workbook) {
    let worksheet: &mut Worksheet = workbook.add_worksheet();
    let title_format = Format::new().set_bold().set_align(FormatAlign::Center);
    let calculated_expenses_list: Vec<CalculatedExpense> =
        repository::get_calculated_expenses(&section.uid);
    let mut row: u32 = 2;
    let formula_children_string: &str = "=$B3";
    let formula_adults_string: &str = "=$B4";
    let formula_children =
        Formula::new(formula_children_string).set_result(section.members_count.to_string());
    let formula_adults =
        Formula::new(formula_adults_string).set_result(section.adults_count.to_string());

    let _ = worksheet
        .set_name(&section.title)
        .expect("Impossible to set the sheet's name");

    let _ = worksheet.merge_range(0, 0, 0, 6, &section.title, &title_format);

    let _ = worksheet.write(row, 0, "Enfants/Ados:");
    let _ = worksheet.write(row, 1, section.members_count);
    row += 1;

    let _ = worksheet.write(row, 0, "Chefs:");
    let _ = worksheet.write(row, 1, section.adults_count);

    row += 2;
    let _ = worksheet.write(row, 0, "Libellé");
    let _ = worksheet.write(row, 1, "Prix unitaire");
    let _ = worksheet.write(row, 2, "Enfants/Ados");
    let _ = worksheet.write(row, 3, "Chefs");
    let _ = worksheet.write(row, 4, "%");
    let _ = worksheet.write(row, 5, "Commentaires");
    let _ = worksheet.write(row, 6, "Total");
    row += 1;

    for expense in calculated_expenses_list {
        let unit_price = match expense.expenses_instances_unit_price {
            Some(val) => val,
            None => expense.expenses_unit_price.unwrap(),
        };

        let rate = match expense.expenses_instances_rate {
            Some(val) => val,
            None => expense.expenses_rate.unwrap(),
        };

        let formula_row = row + 1;

        let formula_total_string = format!(
            "=B{}*(C{}+D{})*(E{}/100)",
            formula_row, formula_row, formula_row, formula_row
        );

        let result: f32 = expense.total_applyed_price.unwrap();
        let formula_total =
            Formula::new(formula_total_string.as_str()).set_result(result.to_string());

        let _ = worksheet.write(row, 0, expense.title_expense.clone());
        let _ = worksheet.write(row, 1, unit_price);

        if expense.expenses_instances_units.is_some() {
            let _ = worksheet.write(row, 2, expense.expenses_instances_units.unwrap());
        } else {
            let _ = worksheet.write_formula(row, 2, &formula_children);
        }

        if expense.expenses_instances_units_adults.is_some() {
            let _ = worksheet.write(row, 3, expense.expenses_instances_units_adults.unwrap());
        } else {
            let _ = worksheet.write_formula(row, 3, &formula_adults);
        }

        let _ = worksheet.write(row, 4, rate);
        let _ = worksheet.write(row, 5, expense.comments.clone());
        let _ = worksheet.write_formula(row, 6, formula_total);

        row += 1
    }

    let _ = worksheet.autofit();
}
