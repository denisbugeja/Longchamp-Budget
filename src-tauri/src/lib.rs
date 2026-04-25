//! Library for the Longchamp Budget application.
//!
//! This module contains the Tauri commands and the main entry point for the application.

mod helper;
mod repository;

use tauri::{AppHandle, Runtime};

use crate::helper::struct_to_json;

/// The build mode of the application (debug or release).
#[cfg(debug_assertions)]
const BUILD_MODE: &str = "debug";

/// The build mode of the application (debug or release).
#[cfg(not(debug_assertions))]
const BUILD_MODE: &str = "release";

/// Updates the database file path and optionally erases the existing file.
///
/// # Arguments
///
/// * `path` - The new path to the database file.
/// * `erase_if_exists` - Whether to erase the file if it already exists.
#[tauri::command]
fn update_db_path(path: &str, erase_if_exists: bool) {
    repository::update_db_file_path(path, erase_if_exists);
}

// Section part

/// Loads the list of all sections as a JSON string.
#[tauri::command]
fn section_list_load() -> String {
    helper::vec_to_json(repository::section_list())
}

/// Inserts a new section into the database.
///
/// # Arguments
///
/// * `title` - The title of the section.
/// * `color` - The hex color code for the section.
/// * `members_count` - The number of members in the section.
/// * `adults_count` - The number of adults in the section.
#[tauri::command]
fn insert_new_section(title: &str, color: &str, members_count: i32, adults_count: i32) {
    repository::insert_new_section(title, color, members_count, adults_count);
}

/// Deletes a section from the database by its UID.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the section.
#[tauri::command]
fn delete_section(uid: &str) {
    repository::delete_section(uid);
}

/// Updates an existing section in the database.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the section.
/// * `title` - The new title of the section.
/// * `color` - The new hex color code for the section.
/// * `members_count` - The new number of members in the section.
/// * `adults_count` - The new number of adults in the section.
#[tauri::command]
fn update_section(uid: &str, title: &str, color: &str, members_count: i32, adults_count: i32) {
    repository::update_section(uid, title, color, members_count, adults_count);
}

// Expense part

/// Loads the list of all expenses as a JSON string.
#[tauri::command]
fn expense_list_load() -> String {
    helper::vec_to_json(repository::expense_list())
}

/// Inserts a new expense into the database and associates it with sections.
///
/// # Arguments
///
/// * `title` - The title of the expense.
/// * `description` - The description of the expense.
/// * `rate` - The rate for the expense (as a string).
/// * `unit_price` - The unit price for the expense (as a string).
/// * `section_list` - A JSON array of section UIDs to associate with the expense.
#[tauri::command]
fn insert_new_expense(
    title: &str,
    description: &str,
    rate: &str,
    unit_price: &str,
    section_list: &str,
) {
    let vec_section_list: Vec<&str> = helper::json_to_vec(section_list);
    repository::insert_new_expense(title, description, rate, unit_price, vec_section_list);
}

/// Updates an existing expense in the database.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the expense.
/// * `title` - The new title.
/// * `description` - The new description.
/// * `rate` - The new rate (as a string).
/// * `unit_price` - The new unit price (as a string).
#[tauri::command]
fn update_expense(uid: &str, title: &str, description: &str, rate: &str, unit_price: &str) {
    repository::update_expense(uid, title, description, rate, unit_price);
}

/// Updates the display order of expenses.
///
/// # Arguments
///
/// * `expense_list` - A JSON array of expense UIDs in the desired order.
#[tauri::command]
fn update_expense_order(expense_list: &str) {
    let vec_expense_list: Vec<&str> = helper::json_to_vec(expense_list);
    repository::update_expense_order(vec_expense_list);
}

/// Updates the display order of sections.
///
/// # Arguments
///
/// * `section_list` - A JSON array of section UIDs in the desired order.
#[tauri::command]
fn update_section_order(section_list: &str) {
    let vec_section_list: Vec<&str> = helper::json_to_vec(section_list);
    repository::update_section_order(vec_section_list);
}

/// Updates the display order of expense instances.
///
/// # Arguments
///
/// * `expense_instance_list` - A JSON array of expense instance UIDs in the desired order.
#[tauri::command]
fn update_expense_instance_order(expense_instance_list: &str) {
    let vec_expense_instance_list: Vec<&str> = helper::json_to_vec(expense_instance_list);
    repository::update_expense_instance_order(vec_expense_instance_list);
}

/// Updates the association between an expense and multiple sections.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the expense.
/// * `section_list` - A JSON array of section UIDs.
#[tauri::command]
fn update_expense_section_association(uid: &str, section_list: &str) {
    let vec_section_list: Vec<&str> = helper::json_to_vec(section_list);
    repository::update_expense_section_association(uid, vec_section_list);
}

/// Gets section-expense associations from all expense instances as a JSON string.
#[tauri::command]
fn get_section_expense_from_expenses_instances() -> String {
    helper::vec_to_json(repository::get_section_expense_from_expenses_instances())
}

/// Gets section-expense associations for a specific section from its expense instances.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_section_expense_from_expenses_instances_section(section_uid: &str) -> String {
    helper::vec_to_json(
        repository::get_section_expense_from_expenses_instances_section(section_uid),
    )
}

/// Gets all section-expense associations as a JSON string.
#[tauri::command]
fn get_section_expense() -> String {
    helper::vec_to_json(repository::get_section_expense())
}

/// Gets section-expense associations for a specific section and expense instance.
///
/// # Arguments
///
/// * `section_uid` - The UID of the section.
/// * `expense_uid` - The UID of the expense.
#[tauri::command]
fn get_section_expense_from_instance(section_uid: &str, expense_uid: &str) -> String {
    helper::struct_to_json(repository::get_section_expense_from_instance(
        section_uid,
        expense_uid,
    ))
}

/// Gets all section-expense associations for a specific expense across all instances.
///
/// # Arguments
///
/// * `expense_uid` - The unique identifier of the expense.
#[tauri::command]
fn get_section_expense_from_instances_by_expense(expense_uid: &str) -> String {
    helper::vec_to_json(repository::get_section_expense_from_instances_wrapper(
        expense_uid,
    ))
}

/// Deletes an expense from the database by its UID.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the expense.
#[tauri::command]
fn delete_expense(uid: &str) {
    repository::delete_expense(uid);
}

/// Updates the members count for a section.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the section.
/// * `members_count` - The new number of members.
#[tauri::command]
fn update_members_count(uid: &str, members_count: i32) {
    repository::update_members_count(uid, members_count);
}

/// Updates the adults count for a section.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the section.
/// * `adults_count` - The new number of adults.
#[tauri::command]
fn update_adults_count(uid: &str, adults_count: i32) {
    repository::update_adults_count(uid, adults_count);
}

/// Adds a new expense instance for a specific section and expense.
///
/// # Arguments
///
/// * `section_uid` - The UID of the section.
/// * `expense_id` - The UID of the expense.
#[tauri::command]
fn add_expense_instance(section_uid: &str, expense_id: &str) {
    repository::add_expense_instance(section_uid, expense_id);
}

/// Gets calculated expenses for a specific section as a JSON string.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_calculated_expenses(section_uid: &str) -> String {
    helper::vec_to_json(repository::get_calculated_expenses(section_uid))
}

/// Gets the members count for a specific section.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_members_count(section_uid: &str) -> i32 {
    repository::get_members_count(section_uid)
}

/// Gets the adults count for a specific section.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_adults_count(section_uid: &str) -> i32 {
    repository::get_adults_count(section_uid)
}

/// Updates an existing expense instance.
///
/// # Arguments
///
/// * `uid_expense_instance` - The unique identifier of the expense instance.
/// * `unit_price` - The unit price (as a string).
/// * `number` - The number of units (as a string).
/// * `units` - The custom units (as a string).
/// * `units_adults` - The custom units for adults (as a string).
/// * `rate` - The custom rate (as a string).
/// * `comments` - Optional comments.
#[tauri::command]
fn update_expense_instance(
    uid_expense_instance: &str,
    unit_price: &str,
    number: &str,
    units: &str,
    units_adults: &str,
    rate: &str,
    comments: &str,
) {
    repository::update_expense_instance(
        uid_expense_instance,
        unit_price,
        number,
        units,
        units_adults,
        rate,
        comments,
    );
}

/// Deletes an expense instance by its UID.
///
/// # Arguments
///
/// * `uid_expense_instance` - The unique identifier of the expense instance.
#[tauri::command]
fn delete_expense_instance(uid_expense_instance: &str) {
    repository::delete_expense_instance(uid_expense_instance);
}

/// Copies an existing expense instance.
///
/// # Arguments
///
/// * `uid_expense_instance` - The unique identifier of the expense instance to copy.
#[tauri::command]
fn copy_expense_instance(uid_expense_instance: &str) {
    repository::copy_expense_instance(uid_expense_instance);
}

/// Gets calculated expenses for the entire group as a JSON string.
#[tauri::command]
fn get_group_calculated_expenses() -> String {
    helper::vec_to_json(repository::get_group_calculated_expenses())
}

/// Gets the sum of calculated expenses for the entire group as a JSON string.
#[tauri::command]
fn get_group_sum_calculated_expenses() -> String {
    helper::struct_to_json(repository::get_group_sum_calculated_expenses())
}

/// Gets the sum of calculated expenses only for the group as a JSON string.
#[tauri::command]
fn get_group_only_sum_calculated_expenses() -> String {
    helper::struct_to_json(repository::get_group_only_sum_calculated_expenses())
}

/// Gets the sum of calculated expenses for a specific section as a JSON string.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_sum_calculated_expenses(section_uid: &str) -> String {
    helper::struct_to_json(repository::get_sum_calculated_expenses(section_uid))
}

/// Gets the total expense per member for a specific section as a JSON string.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_total_per_member(section_uid: &str) -> String {
    helper::struct_to_json(repository::get_total_per_member(section_uid))
}

/// Gets the count of a specific expense for a section from its instances.
///
/// # Arguments
///
/// * `section_uid` - The UID of the section.
/// * `expense_uid` - The UID of the expense.
#[tauri::command]
fn get_section_expense_cnt_from_instance(section_uid: &str, expense_uid: &str) -> String {
    repository::get_section_expense_cnt_from_instance(section_uid, expense_uid).to_string()
}

/// Gets the section-expense association for a specific section and expense.
///
/// # Arguments
///
/// * `section_uid` - The UID of the section.
/// * `expense_uid` - The UID of the expense.
#[tauri::command]
fn get_section_expense_from_association(section_uid: &str, expense_uid: &str) -> String {
    helper::struct_to_json(repository::get_section_expense_from_association(
        section_uid,
        expense_uid,
    ))
}

/// Gets section-expense associations for a section from its expense instances as a JSON string.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_section_expense_from_expenses_instances_and_section(section_uid: &str) -> String {
    helper::struct_to_json(
        repository::get_section_expense_from_expenses_instances_and_section(section_uid),
    )
}

/// Reads an asset file and returns its content as a string.
///
/// # Arguments
///
/// * `app` - The Tauri AppHandle.
/// * `path` - The path to the asset.
#[tauri::command]
fn read_asset<R: Runtime>(app: AppHandle<R>, path: &str) -> String {
    let path_string = String::from(path);
    let asset = app
        .asset_resolver()
        .get(path_string)
        .expect("Impossible de trouver l'asset");

    String::from_utf8(asset.bytes.to_vec()).expect("Echec de conversion en UTF-8")
}

/// Generates an XLS file based on the database data.
#[tauri::command]
fn generate_xls_file() {
    helper::generate_xls_file();
}

/// Gets the global database file path.
#[tauri::command]
fn get_global_file_path() -> String {
    repository::get_global_file_path()
}

/// Loads the list of all QFs (Quotient Familial) as a JSON string.
#[tauri::command]
fn fq_list_load() -> String {
    helper::vec_to_json(repository::fq_list())
}

/// Loads the list of QFs associated with a specific section as a JSON string.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn fq_section_list_load(section_uid: &str) -> String {
    helper::vec_to_json(repository::fq_section_list_load(section_uid))
}

/// Inserts a new QF into the database.
///
/// # Arguments
///
/// * `title` - The title of the QF.
/// * `coeff` - The multiplier coefficient (as a string).
/// * `national_contribution` - The national contribution amount (as a string).
/// * `online_commission_rate` - The online commission rate (as a string).
/// * `online_commission_fees` - The online commission fixed fees (as a string).
#[tauri::command]
fn insert_new_fq(
    title: &str,
    coeff: &str,
    national_contribution: &str,
    online_commission_rate: &str,
    online_commission_fees: &str,
) {
    repository::insert_new_fq(
        title,
        coeff,
        national_contribution,
        online_commission_rate,
        online_commission_fees,
    );
}

/// Updates an existing QF in the database.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the QF.
/// * `title` - The new title.
/// * `coeff` - The new coefficient (as a string).
/// * `national_contribution` - The new national contribution (as a string).
/// * `online_commission_rate` - The new online commission rate (as a string).
/// * `online_commission_fees` - The new online commission fees (as a string).
#[tauri::command]
fn update_fq(
    uid: &str,
    title: &str,
    coeff: &str,
    national_contribution: &str,
    online_commission_rate: &str,
    online_commission_fees: &str,
) {
    repository::update_fq(
        uid,
        title,
        coeff,
        national_contribution,
        online_commission_rate,
        online_commission_fees,
    );
}

/// Deletes a QF from the database by its UID.
///
/// # Arguments
///
/// * `uid` - The unique identifier of the QF.
#[tauri::command]
fn delete_fq(uid: &str) {
    repository::delete_fq(uid);
}

/// Updates the display order of QFs.
///
/// # Arguments
///
/// * `fq_list` - A JSON array of QF UIDs in the desired order.
#[tauri::command]
fn update_fq_order(fq_list: &str) {
    let vec_fq_list: Vec<&str> = helper::json_to_vec(fq_list);
    repository::update_fq_order(vec_fq_list);
}

/// Updates the members count for a specific QF in a section.
///
/// # Arguments
///
/// * `section_uid` - The UID of the section.
/// * `fq_uid` - The UID of the QF.
/// * `members_count` - The new number of members.
#[tauri::command]
fn update_fq_section_members_count(section_uid: &str, fq_uid: &str, members_count: i32) {
    repository::update_fq_section_members_count(section_uid, fq_uid, members_count);
}

/// Gets the total number of members across all QFs for a specific section.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_members_fq_count_by_section(section_uid: &str) -> String {
    repository::get_members_fq_count_by_section(section_uid).to_string()
}

/// Gets the members count for all sections as a JSON string.
#[tauri::command]
fn get_members_fq_count_for_all_sections() -> String {
    struct_to_json(repository::get_members_fq_count_for_all_sections())
}

/// Gets calculated QF values for a specific section as a JSON string.
///
/// # Arguments
///
/// * `section_uid` - The unique identifier of the section.
#[tauri::command]
fn get_fqs_calculated_by_section(section_uid: &str) -> String {
    helper::struct_to_json(repository::get_fqs_calculated_by_section(section_uid))
}

/// Gets the current build mode (debug or release).
#[tauri::command]
fn get_build_mode() -> String {
    BUILD_MODE.to_string()
}

/// The main entry point for the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            read_asset,
            update_db_path,
            section_list_load,
            insert_new_section,
            delete_section,
            update_section,
            expense_list_load,
            insert_new_expense,
            get_section_expense_from_expenses_instances,
            get_section_expense,
            get_section_expense_from_instances_by_expense,
            update_expense_section_association,
            update_expense,
            delete_expense,
            update_members_count,
            update_adults_count,
            add_expense_instance,
            get_calculated_expenses,
            get_section_expense_from_expenses_instances_section,
            get_members_count,
            get_adults_count,
            update_expense_instance,
            delete_expense_instance,
            copy_expense_instance,
            get_group_calculated_expenses,
            get_group_sum_calculated_expenses,
            get_group_only_sum_calculated_expenses,
            get_sum_calculated_expenses,
            get_total_per_member,
            get_section_expense_cnt_from_instance,
            get_section_expense_from_instance,
            get_section_expense_from_association,
            get_section_expense_from_expenses_instances_and_section,
            generate_xls_file,
            get_global_file_path,
            update_expense_order,
            update_section_order,
            update_expense_instance_order,
            fq_list_load,
            fq_section_list_load,
            insert_new_fq,
            delete_fq,
            update_fq,
            update_fq_order,
            update_fq_section_members_count,
            get_members_fq_count_by_section,
            get_fqs_calculated_by_section,
            get_members_fq_count_for_all_sections,
            get_build_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error) while running tauri application");
}
