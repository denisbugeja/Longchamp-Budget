use crate::repository;
use rust_xlsxwriter::{
    Color, Format, FormatAlign, FormatBorder, Formula, Note, Workbook, Worksheet,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    pub uid: String,
    pub title: String,
    pub color: String,
    pub members_count: f32,
    pub adults_count: f32,
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
    pub description: Option<String>,
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
    pub expenses_units: Option<f32>,
    pub expenses_units_adults: Option<f32>,
    pub expenses_unit_price: Option<f32>,
    pub expenses_rate: Option<f32>,
    pub expenses_instances_number: Option<f32>,
    pub expenses_instances_units: Option<f32>,
    pub expenses_instances_units_adults: Option<f32>,
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
    pub expenses_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SumExpenseInstance {
    pub sum_unit: f32,
    pub sum_total: f32,
}

//TODO Ajouter taux pour frais de commision en ligne
//TODO Ajouter montant fixe pour frais de commision en ligne
// Actuellement =0,4+(0,8%*G90)
#[derive(Debug, Serialize, Deserialize)]
pub struct Fq {
    pub uid: String,
    pub title: String,
    pub coeff: f32,
    pub national_contribution: f32,
    pub online_commission_rate: f32,
    pub online_commission_fees: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FqSection {
    pub uid_fq: String,
    pub uid_section: String,
    pub members_count: f32,
    pub title_section: String,
    pub title_fq: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FqTotal {
    pub title_section: String,
    pub title_fq: String,
    pub uid_fq: String,
    pub uid_section: String,
    pub declared_unit_price: f32,
    pub declared_group_unit_price: f32,
    pub coeff: f32,
    pub calculated_unit_price_with_coeff: f32,
    pub group_calculated_unit_price: f32,
    pub total_group_member_price: f32,
    pub national_contribution: f32,
    pub total_member_price: f32,
    pub national_commission: f32,
    pub total: f32,
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

    let group_expense_list: Vec<CalculatedExpense> = repository::get_group_calculated_expenses();

    for section in section_list {
        handle_worksheet(&section, &mut workbook, &group_expense_list);
    }

    let fq_list = repository::fq_list();
    if !fq_list.is_empty() {
        add_fq_data_to_work_book(&mut workbook);
    }

    let _ = workbook.save(final_path);
}

fn handle_worksheet(
    section: &Section,
    workbook: &mut Workbook,
    group_expense_list: &Vec<CalculatedExpense>,
) {
    let worksheet: &mut Worksheet = workbook.add_worksheet();
    let title_format = Format::new().set_bold().set_align(FormatAlign::Center);
    let border_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black);

    let border_bold_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_bold();

    let border_bold_center_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Center)
        .set_num_format("0.00")
        .set_bold();

    let border_number_right_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Right)
        .set_num_format("0.00");

    let border_bold_number_right_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Right)
        .set_num_format("0.00")
        .set_bold();

    let calculated_expenses_list: Vec<CalculatedExpense> =
        repository::get_calculated_expenses(&section.uid);
    let mut row: u32 = 2;
    let mut row_total_unite: u32 = 2;
    let formula_children_string: &str = "=$B$3";
    let formula_adults_string: &str = "=$B$4";
    let formula_children =
        Formula::new(formula_children_string).set_result(section.members_count.to_string());
    let formula_adults =
        Formula::new(formula_adults_string).set_result(section.adults_count.to_string());

    let mut title_tab = format!("Unité {}", &section.title);
    if "group" == section.uid {
        title_tab = format!("Agrégat {}", &section.title);
    }

    let _ = worksheet
        .set_name(title_tab)
        .expect("Impossible to set the sheet's name");

    let _ = worksheet.merge_range(0, 0, 0, 7, &section.title, &title_format);

    let _ = worksheet.write_with_format(row, 0, "Enfants/Ados:", &border_bold_format);
    let _ = worksheet.write_number_with_format(row, 1, section.members_count, &border_format);
    row += 1;

    let _ = worksheet.write_with_format(row, 0, "Chefs:", &border_bold_format);
    let _ = worksheet.write_number_with_format(row, 1, section.adults_count, &border_format);

    row += 2;
    let _ = worksheet.write_with_format(row, 0, "Libellé", &border_bold_format);
    let _ = worksheet.write_with_format(row, 1, "Prix unitaire", &border_bold_format);
    let _ = worksheet.write_with_format(row, 2, "Occurrences", &border_bold_format);

    let _ = worksheet.write_with_format(row, 3, "Enfants/Ados", &border_bold_format);
    let _ = worksheet.write_with_format(row, 4, "Chefs", &border_bold_format);
    let _ = worksheet.write_with_format(row, 5, "%", &border_bold_format);
    let _ = worksheet.write_with_format(row, 6, "Commentaires", &border_bold_format);
    let _ = worksheet.write_with_format(row, 7, "Total", &border_bold_format);
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
            "=ROUND((B{formula_row}*(C{formula_row}*(D{formula_row}+E{formula_row}))*(F{formula_row}/100)),2)"
        );

        let result: f32 = expense.total_applyed_price.unwrap();
        let formula_total =
            Formula::new(formula_total_string.as_str()).set_result(result.to_string());

        let _ = worksheet.write_with_format(row, 0, expense.title_expense.clone(), &border_format);

        if !expense.expenses_description.clone().unwrap().is_empty() {
            let _ = worksheet.insert_note(
                row,
                0,
                &Note::new(expense.expenses_description.clone().unwrap()),
            );
        }

        let _ = worksheet.write_number_with_format(row, 1, unit_price, &border_number_right_format);

        let _ = worksheet.write_number_with_format(
            row,
            2,
            expense.expenses_instances_number.unwrap(),
            &border_format,
        );

        if expense.expenses_instances_units.is_some() {
            let _ = worksheet.write_number_with_format(
                row,
                3,
                expense.expenses_instances_units.unwrap(),
                &border_format,
            );
        } else {
            let _ = worksheet.write_formula_with_format(row, 3, &formula_children, &border_format);
        }

        if expense.expenses_instances_units_adults.is_some() {
            let _ = worksheet.write_number_with_format(
                row,
                4,
                expense.expenses_instances_units_adults.unwrap(),
                &border_format,
            );
        } else {
            let _ = worksheet.write_formula_with_format(row, 4, &formula_adults, &border_format);
        }

        let _ = worksheet.write_number_with_format(row, 5, rate, &border_format);
        let _ = worksheet.write_with_format(row, 6, expense.comments.clone(), &border_format);
        let _ = worksheet.write_formula_with_format(row, 7, formula_total, &border_format);

        row += 1;
    }

    if !calculated_expenses_list.is_empty() {
        let sum_calculated = repository::get_sum_calculated_expenses(&section.uid);
        let formula_sum = Formula::new(format!("=SUM(H{first_excel_row}:H{row})"))
            .set_result(sum_calculated.sum_total.to_string());

        let _ = worksheet.merge_range(row, 4, row, 6, "Total Unité", &border_format);
        let _ = worksheet.write_formula_with_format(
            row,
            7,
            &formula_sum,
            &border_bold_number_right_format,
        );

        row += 1;
        row_total_unite = row + 1;
        let formula_sum_units = Formula::new(format!("=IF($B$3=0,0,ROUND((H{row}/$B$3),2))"))
            .set_result(sum_calculated.sum_unit.to_string());

        let mut total_label = String::from("Total Unité par enfant");
        if "group" == section.uid && group_expense_list.is_empty() {
            total_label = String::from("Total Groupe par enfant");
        }

        let _ = worksheet.merge_range(row, 4, row, 6, &total_label, &border_format);
        let _ = worksheet.write_formula_with_format(
            row,
            7,
            &formula_sum_units,
            &border_bold_number_right_format,
        );

        if "group" != section.uid {
            row += 1;
            let sum_calculated_group: SumExpenseInstance =
                repository::get_group_sum_calculated_expenses();
            let _ =
                worksheet.merge_range(row, 4, row, 6, "Total Groupe par enfant", &border_format);
            let _ = worksheet.write_number_with_format(
                row,
                7,
                sum_calculated_group.sum_unit,
                &border_bold_number_right_format,
            );

            row += 1;
            let total_per_member = repository::get_total_per_member(&section.uid);
            let formula_sum_total = Formula::new(format!("=SUM(H{}:H{})", row - 1, row))
                .set_result(total_per_member.sum_unit.to_string());
            let _ = worksheet.merge_range(row, 4, row, 6, "Total par enfant", &border_format);
            let _ = worksheet.write_formula_with_format(
                row,
                7,
                &formula_sum_total,
                &border_bold_number_right_format,
            );
        }
    }

    if "group" == section.uid {
        let group_sum_expense_instance: SumExpenseInstance =
            repository::get_group_only_sum_calculated_expenses();

        if !&group_expense_list.is_empty() {
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

            let sum_row_begin: u32 = row + 2;
            for group_expense in group_expense_list {
                row += 1;
                let _ = worksheet.write_with_format(
                    row,
                    0,
                    group_expense.title_expense.clone(),
                    &border_format,
                );

                if !group_expense
                    .expenses_description
                    .clone()
                    .unwrap()
                    .is_empty()
                {
                    let _ = worksheet.insert_note(
                        row,
                        0,
                        &Note::new(group_expense.expenses_description.clone().unwrap()),
                    );
                }

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

                let formula_group_unit =
                    Formula::new(format!("=IF($B$3=0,0,ROUND((F{}/$B$3),2))", row + 1)).set_result(
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
                    &border_number_right_format,
                );

                let _ = worksheet.write_number_with_format(
                    row,
                    5,
                    group_expense.group_applyed_total_price.unwrap(),
                    &border_number_right_format,
                );
            }

            row += 1;
            let sum_row_end: u32 = row;
            let mut total_label_ratio = String::from("Cotisation répartie par enfant");
            if "group" == section.uid && calculated_expenses_list.is_empty() {
                total_label_ratio = String::from("Total Groupe par enfant");
            }
            let row_total_rated_group = row + 1;
            let sum_calculated_group: SumExpenseInstance =
                repository::get_group_sum_calculated_expenses();
            let _ = worksheet.merge_range(row, 1, row, 3, &total_label_ratio, &border_format);

            let formula_total_calculated_group =
                Formula::new(format!("=ROUND(SUM(E{sum_row_begin}:E{sum_row_end}),2)")).set_result(
                    group_sum_expense_instance
                        .sum_unit
                        .to_string()
                        .replace(".", ","),
                );
            let _ = worksheet.write_formula_with_format(
                row,
                4,
                formula_total_calculated_group,
                &border_bold_number_right_format,
            );

            if !&calculated_expenses_list.is_empty() {
                row += 3;
                let formula_total_group =
                    Formula::new(format!("=$H${row_total_unite}+$E${row_total_rated_group}"))
                        .set_result(sum_calculated_group.sum_unit.to_string());
                let _ = worksheet.write_with_format(
                    row,
                    0,
                    "Total Groupe par enfant",
                    &border_bold_format,
                );
                let _ = worksheet.write_formula_with_format(
                    row,
                    1,
                    formula_total_group,
                    &border_bold_number_right_format,
                );
            }
        }
    }

    // FQ
    if !calculated_expenses_list.is_empty() {
        let fq_list: Vec<FqTotal> = repository::get_fqs_calculated_by_section(&section.uid);
        if !fq_list.is_empty() {
            row += 4;
            let _ = worksheet.merge_range(row, 0, 0, 7, "Prise en charge des QF", &title_format);

            row += 2;
            let _ = worksheet.write_with_format(row, 0, "QF", &border_bold_format);
            let _ = worksheet.write_with_format(
                row,
                1,
                "Coefficient multiplicateur",
                &border_bold_format,
            );

            if "group" != section.uid {
                let _ =
                    worksheet.write_with_format(row, 2, "Cotisation unité", &border_bold_format);
                let _ = worksheet.write_with_format(
                    row,
                    3,
                    "Total cotisations groupe + unité",
                    &border_bold_format,
                );
                let _ = worksheet.write_with_format(
                    row,
                    4,
                    "Cotisation nationale",
                    &border_bold_format,
                );
                let _ = worksheet.write_with_format(row, 5, "Total", &border_bold_format);
                let _ = worksheet.insert_note(
                    row,
                    5,
                    &Note::new("Le total comprend les frais de commission en ligne"),
                );
            } else {
                let _ =
                    worksheet.write_with_format(row, 2, "Cotisation groupe", &border_bold_format);
            }

            for fq in fq_list {
                row += 1;
                let _ =
                    worksheet.write_with_format(row, 0, fq.title_fq, &border_number_right_format);
                let _ = worksheet.write_with_format(row, 1, fq.coeff, &border_number_right_format);
                let _ = worksheet.write_with_format(
                    row,
                    2,
                    fq.calculated_unit_price_with_coeff,
                    &border_number_right_format,
                );
                if "group" != section.uid {
                    let _ = worksheet.write_with_format(
                        row,
                        3,
                        fq.total_group_member_price,
                        &border_number_right_format,
                    );
                    let _ = worksheet.write_with_format(
                        row,
                        4,
                        fq.national_contribution,
                        &border_number_right_format,
                    );
                    let _ = worksheet.write_with_format(
                        row,
                        5,
                        fq.total,
                        &border_bold_number_right_format,
                    );
                }
            }
        }
    }

    let _ = worksheet.autofit();
}

fn add_fq_data_to_work_book(workbook: &mut Workbook) {
    let fq_list = repository::get_calculated_fqs_total_without_group();
    if fq_list.is_empty() {
        return;
    }

    let title_format = Format::new().set_bold().set_align(FormatAlign::Center);
    let border_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black);

    let border_bold_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_bold();

    let border_bold_center_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Center)
        .set_num_format("0.00")
        .set_bold();

    let border_number_right_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Right)
        .set_num_format("0.00");

    let border_bold_number_right_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
        .set_align(FormatAlign::Right)
        .set_num_format("0.00")
        .set_bold();

    let worksheet: &mut Worksheet = workbook
        .add_worksheet()
        .set_name("QF")
        .expect("Impossible to set the sheet's name");

    let _ = worksheet.merge_range(0, 0, 0, 11, "QF", &title_format);

    let mut row = 2;
    let mut formula_row = row + 1;
    let _ = worksheet.write_with_format(row, 0, "Unité", &border_bold_center_format);
    let _ = worksheet.write_with_format(row, 1, "QF", &border_bold_center_format);
    let _ = worksheet.write_with_format(
        row,
        2,
        "Cotisation unité moyenne pondérée",
        &border_bold_center_format,
    );
    let _ = worksheet.write_with_format(
        row,
        3,
        "Cotisation groupe moyenne pondérée",
        &border_bold_center_format,
    );
    let _ = worksheet.write_with_format(
        row,
        4,
        "Coefficient multiplicateur QF",
        &border_bold_center_format,
    );
    let _ =
        worksheet.write_with_format(row, 5, "Montant unité calculé", &border_bold_center_format);
    let _ =
        worksheet.write_with_format(row, 6, "Montant groupe calculé", &border_bold_center_format);
    let _ =
        worksheet.write_with_format(row, 7, "Montant total calculé", &border_bold_center_format);
    let _ =
        worksheet.write_with_format(row, 8, "Contribution nationale", &border_bold_center_format);
    let _ = worksheet.write_with_format(
        row,
        9,
        "Total groupe + national",
        &border_bold_center_format,
    );
    let _ = worksheet.write_with_format(row, 10, "Frais de commision", &border_bold_center_format);
    let _ = worksheet.write_with_format(row, 11, "Cotisation totale", &border_bold_center_format);

    for fq in fq_list {
        row += 1;
        formula_row = row + 1;

        let _ = worksheet.write_with_format(row, 0, fq.title_section, &border_format);
        let _ = worksheet.write_with_format(row, 1, fq.title_fq, &border_format);
        let _ = worksheet.write_with_format(
            row,
            2,
            fq.declared_unit_price,
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            3,
            fq.declared_group_unit_price,
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(row, 4, fq.coeff, &border_number_right_format);
        let _ = worksheet.write_with_format(
            row,
            5,
            Formula::new(format!("=ROUND(C{formula_row}*E{formula_row},2)")).set_result(
                fq.calculated_unit_price_with_coeff
                    .to_string()
                    .replace(".", ","),
            ),
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            6,
            Formula::new(format!("=ROUND(D{formula_row}*E{formula_row},2)"))
                .set_result(fq.group_calculated_unit_price.to_string().replace(".", ",")),
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            7,
            Formula::new(format!("=ROUND(F{formula_row}+G{formula_row},2)"))
                .set_result(fq.total_group_member_price.to_string().replace(".", ",")),
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            8,
            fq.national_contribution,
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            9,
            Formula::new(format!("=ROUND(H{formula_row}+I{formula_row},2)"))
                .set_result(fq.total_member_price.to_string().replace(".", ",")),
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            10,
            fq.national_commission,
            &border_number_right_format,
        );
        let _ = worksheet.write_with_format(
            row,
            11,
            Formula::new(format!("=ROUND(J{formula_row}+K{formula_row},2)"))
                .set_result(fq.total.to_string().replace(".", ",")),
            &border_bold_number_right_format,
        );
    }
}
