use crate::repository;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Formula, Workbook, Worksheet};
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
    let border_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black);

    let border_bold_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_bold();

    let border_bold_right_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Right)
        .set_bold();

    let border_bold_center_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Center)
        .set_bold();

    let border_bold_number_right_format =
        border_bold_right_format.clone().set_num_format("#,##0.00");

    let calculated_expenses_list: Vec<CalculatedExpense> =
        repository::get_calculated_expenses(&section.uid);
    let mut row: u32 = 2;
    let mut row_total_unite: u32 = 2;
    let mut row_total_rated_group: u32 = 2;
    let formula_children_string: &str = "=B3";
    let formula_adults_string: &str = "=B4";
    let formula_children =
        Formula::new(formula_children_string).set_result(section.members_count.to_string());
    let formula_adults =
        Formula::new(formula_adults_string).set_result(section.adults_count.to_string());

    let _ = worksheet
        .set_name(&section.title)
        .expect("Impossible to set the sheet's name");

    let _ = worksheet.merge_range(0, 0, 0, 6, &section.title, &title_format);

    let _ = worksheet.write_with_format(row, 0, "Enfants/Ados:", &border_bold_format);
    let _ = worksheet.write_number_with_format(row, 1, section.members_count, &border_format);
    row += 1;

    let _ = worksheet.write_with_format(row, 0, "Chefs:", &border_bold_format);
    let _ = worksheet.write_number_with_format(row, 1, section.adults_count, &border_format);

    row += 2;
    let _ = worksheet.write_with_format(row, 0, "Libellé", &border_bold_format);
    let _ = worksheet.write_with_format(row, 1, "Prix unitaire", &border_bold_format);
    let _ = worksheet.write_with_format(row, 2, "Enfants/Ados", &border_bold_format);
    let _ = worksheet.write_with_format(row, 3, "Chefs", &border_bold_format);
    let _ = worksheet.write_with_format(row, 4, "%", &border_bold_format);
    let _ = worksheet.write_with_format(row, 5, "Commentaires", &border_bold_format);
    let _ = worksheet.write_with_format(row, 6, "Total", &border_bold_format);
    row += 1;

    let first_excel_row = row + 1;
    for expense in &calculated_expenses_list {
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
            "=ROUND((B{}*(C{}+D{})*(E{}/100)),2)",
            formula_row, formula_row, formula_row, formula_row
        );

        let result: f32 = expense.total_applyed_price.unwrap();
        let formula_total = Formula::new(formula_total_string.as_str())
            .set_result(result.to_string().replace(".", ","));

        let _ = worksheet.write_with_format(row, 0, expense.title_expense.clone(), &border_format);
        let _ = worksheet.write_number_with_format(row, 1, unit_price, &border_format);

        if expense.expenses_instances_units.is_some() {
            let _ = worksheet.write_number_with_format(
                row,
                2,
                expense.expenses_instances_units.unwrap(),
                &border_format,
            );
        } else {
            let _ = worksheet.write_formula_with_format(row, 2, &formula_children, &border_format);
        }

        if expense.expenses_instances_units_adults.is_some() {
            let _ = worksheet.write_number_with_format(
                row,
                3,
                expense.expenses_instances_units_adults.unwrap(),
                &border_format,
            );
        } else {
            let _ = worksheet.write_formula_with_format(row, 3, &formula_adults, &border_format);
        }

        let _ = worksheet.write_number_with_format(row, 4, rate, &border_format);
        let _ = worksheet.write_with_format(row, 5, expense.comments.clone(), &border_format);
        let _ = worksheet.write_formula_with_format(row, 6, formula_total, &border_format);

        row += 1;
    }

    if !calculated_expenses_list.is_empty() {
        let sum_calculated = repository::get_sum_calculated_expenses(&section.uid);
        let formula_sum = Formula::new(format!("=SUM(G{}:G{})", first_excel_row, row))
            .set_result(sum_calculated.sum_total.to_string().replace(".", ","));

        let _ = worksheet.merge_range(row, 3, row, 5, "Total Unité", &border_format);
        let _ = worksheet.write_formula_with_format(
            row,
            6,
            &formula_sum,
            &border_bold_number_right_format,
        );

        row += 1;
        row_total_unite = row + 1;
        let formula_sum_units = Formula::new(format!("=ROUND((G{}/B3),2)", row))
            .set_result(sum_calculated.sum_unit.to_string().replace(".", ","));
        let _ = worksheet.merge_range(row, 3, row, 5, "Total Unité par enfant", &border_format);
        let _ = worksheet.write_formula_with_format(
            row,
            6,
            &formula_sum_units,
            &border_bold_number_right_format,
        );

        if "group" != section.uid {
            row += 1;
            let sum_calculated_group: SumExpenseInstance =
                repository::get_group_sum_calculated_expenses();
            let _ =
                worksheet.merge_range(row, 3, row, 5, "Total Groupe par enfant", &border_format);
            let _ = worksheet.write_number_with_format(
                row,
                6,
                sum_calculated_group.sum_unit,
                &border_bold_number_right_format,
            );

            row += 1;
            let total_per_member = repository::get_total_per_member(&section.uid);
            let formula_sum_total = Formula::new(format!("=SUM(G{}:G{})", row - 1, row))
                .set_result(total_per_member.sum_unit.to_string().replace(".", ","));
            let _ = worksheet.merge_range(row, 3, row, 5, "Total par enfant", &border_format);
            let _ = worksheet.write_formula_with_format(
                row,
                6,
                &formula_sum_total,
                &border_bold_number_right_format,
            );
        }
    }

    if "group" == section.uid {
        let groupe_expense_list: Vec<CalculatedExpense> =
            repository::get_group_calculated_expenses();

        let group_sum_expense_instance: SumExpenseInstance =
            repository::get_group_only_sum_calculated_expenses();

        if !groupe_expense_list.is_empty() {
            row += 3;
            let _ = worksheet.merge_range(
                row,
                0,
                row,
                5,
                "Dépenses partiellement rattachées au groupe",
                &border_bold_center_format,
            );

            row += 1;
            let _ = worksheet.write_with_format(row, 0, "Libellé", &border_bold_format);
            let _ = worksheet.write_with_format(row, 1, "Section/Unité", &border_bold_format);
            let _ = worksheet.write_with_format(row, 2, "Commentaires", &border_bold_format);
            let _ = worksheet.write_with_format(row, 3, "% restant", &border_bold_format);
            let _ =
                worksheet.write_with_format(row, 4, "Prix unitaire calculé", &border_bold_format);
            let _ = worksheet.write_with_format(row, 5, "Prix total", &border_bold_format);

            //TODO refacto total formula in case of empty groupe_expense_list or ratio expense list
            for group_expense in &groupe_expense_list {
                row += 1;
                let _ = worksheet.write_with_format(
                    row,
                    0,
                    group_expense.title_expense.clone(),
                    &border_format,
                );
                let _ = worksheet.write_with_format(
                    row,
                    1,
                    group_expense.title_section.clone(),
                    &border_format,
                );

                let _ = worksheet.write_with_format(
                    row,
                    2,
                    group_expense.comments.clone().unwrap_or_else(String::new),
                    &border_format,
                );

                let _ =
                    worksheet.write_with_format(row, 3, group_expense.group_rate, &border_format);

                let formula_group_unit = Formula::new(format!("=ROUND((F{}/B3),2)", row + 1))
                    .set_result(
                        group_expense
                            .group_applyed_unit_price
                            .unwrap()
                            .to_string()
                            .replace(".", ","),
                    );
                let _ = worksheet.write_formula_with_format(
                    row,
                    4,
                    &formula_group_unit,
                    &border_bold_number_right_format,
                );

                let _ = worksheet.write_number_with_format(
                    row,
                    5,
                    group_expense.group_applyed_total_price.unwrap(),
                    &border_bold_number_right_format,
                );
            }
        }

        row += 1;
        row_total_rated_group = row + 1;
        let sum_calculated_group: SumExpenseInstance =
            repository::get_group_sum_calculated_expenses();
        let _ = worksheet.merge_range(
            row,
            1,
            row,
            3,
            "Cotisation répartie par enfant",
            &border_format,
        );
        let _ = worksheet.write_number_with_format(
            row,
            4,
            group_sum_expense_instance.sum_unit,
            &border_bold_number_right_format,
        );

        row += 3;
        let formula_total_group =
            Formula::new(format!("=G{}+E{}", row_total_unite, row_total_rated_group))
                .set_result(sum_calculated_group.sum_unit.to_string());
        let _ = worksheet.write_with_format(row, 0, "Total Groupe par enfant", &border_bold_format);
        let _ = worksheet.write_formula_with_format(
            row,
            1,
            formula_total_group,
            &border_bold_number_right_format,
        );
    }

    let _ = worksheet.autofit();
}
