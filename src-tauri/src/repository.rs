//! Database repository for the Longchamp Budget application.
//!
//! This module handles all interactions with the SQLite database using `rusqlite`.
//! It includes functions for managing sections, expenses, QFs, and their associations,
//! as well as database initialization and migrations.

use crate::structs::{
    CalculatedExpense, Expense, Fq, FqMembersCount, FqSection, FqTotal, NationalFees, Section,
    SectionExpense, SumExpenseInstance,
};
use rusqlite::types::FromSql;
use rusqlite::{Connection, Row};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Repository for managing database connections and operations.
pub struct Repository {
    connection: Mutex<Option<Connection>>,
    file_path: RwLock<String>,
    cache: RwLock<HashMap<String, String>>,
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository {
    /// Creates a new instance of the Repository.
    pub fn new() -> Self {
        Self {
            connection: Mutex::new(None),
            file_path: RwLock::new(String::from("")),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the database file path.
    pub fn get_file_path(&self) -> String {
        let path = self
            .file_path
            .read()
            .expect("Impossible de lire le chemin du fichier");
        path.clone()
    }

    /// Sets the database file path and initializes the database.
    pub fn set_file_path(&self, str_path: &str, erase_if_exists: bool) {
        let mut real_path = String::from(str_path);
        if !real_path.ends_with(".lb") {
            real_path.push_str(".lb");
        }
        let path = Path::new(&real_path);

        if erase_if_exists && path.exists() {
            fs::remove_file(path).expect("Impossible de supprimer le fichier")
        }

        {
            let mut file_path = self
                .file_path
                .write()
                .expect("Impossible d'écrire le chemin du fichier");
            *file_path = real_path.clone();
        }

        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let conn =
            Connection::open(real_path).expect("Impossible d'ouvrir le fichier de base de données");

        Self::execute_migrations(&conn);

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        *connection_lock = Some(conn);
    }

    /// Inserts a new section into the database.
    pub fn insert_new_section(
        &self,
        title: &str,
        color: &str,
        members_count: i32,
        adults_count: i32,
    ) {
        let existing_sections: Vec<Section> = self.fetch_rows(
            include_str!("sql_queries/insert_new_section/existing_sections.sql"),
            &[title],
        );

        if !existing_sections.is_empty() {
            return;
        }

        self.execute_write_sql(
            include_str!("sql_queries/insert_new_section/insert.sql"),
            (
                Uuid::new_v4().to_string(),
                title,
                color,
                members_count.abs(),
                adults_count.abs(),
            ),
        );
    }

    /// Inserts a new QF category into the database.
    pub fn insert_new_fq(
        &self,
        title: &str,
        coeff: &str,
        national_contribution: &str,
        online_commission_rate: &str,
        online_commission_fees: &str,
    ) {
        let coeff_f32: f32 = coeff
            .parse()
            .expect("Échec de l'analyse du coefficient (coeff) en f32");
        let national_contribution_f32: f32 = national_contribution
            .parse()
            .expect("Échec de l'analyse de la cotisation nationale en f32");
        let online_commission_rate_f32: f32 = online_commission_rate
            .parse()
            .expect("Échec de l'analyse du taux de commission en ligne en f32");
        let online_commission_fees_f32: f32 = online_commission_fees
            .parse()
            .expect("Échec de l'analyse des frais de commission en ligne en f32");

        let existing_fqs: Vec<Fq> = self.fetch_rows(
            include_str!("sql_queries/insert_new_fq/existing_fqs.sql"),
            &[title],
        );

        if !existing_fqs.is_empty() {
            return;
        }

        self.execute_write_sql(
            include_str!("sql_queries/insert_new_fq/insert.sql"),
            (
                Uuid::new_v4().to_string(),
                title,
                coeff_f32,
                national_contribution_f32,
                online_commission_rate_f32,
                online_commission_fees_f32,
            ),
        );
    }

    /// Returns a list of all sections ordered by position.
    pub fn section_list(&self) -> Vec<Section> {
        self.fetch_rows(include_str!("sql_queries/section_list.sql"), [])
    }

    /// Returns a list of all QF categories ordered by position.
    pub fn fq_list(&self) -> Vec<Fq> {
        self.fetch_rows(include_str!("sql_queries/fq_list.sql"), [])
    }

    /// Returns a list of QF categories and their member counts for a specific section.
    pub fn fq_section_list_load(&self, section_uid: &str) -> Vec<FqSection> {
        self.fetch_rows(
            include_str!("sql_queries/fq_section_list_load.sql"),
            &[section_uid],
        )
    }

    /// Returns calculated QF total data for a specific section.
    pub fn get_fqs_calculated_by_section(&self, section_uid: &str) -> Vec<FqTotal> {
        self.fetch_rows(
            include_str!("sql_queries/get_fqs_calculated_by_section.sql"),
            &[section_uid],
        )
    }

    /// Returns the total national fees (contribution and commission) across all sections.
    pub fn get_total_national_cotisation(&self) -> NationalFees {
        let mut result = self.fetch_rows(
            include_str!("sql_queries/get_total_national_cotisation.sql"),
            [],
        );

        result.pop().expect("Aucun frais de cotisation trouvé")
    }

    /// Returns calculated QF total data for all sections except the aggregate group.
    pub fn get_calculated_fqs_total_without_group(&self) -> Vec<FqTotal> {
        self.fetch_rows(
            include_str!("sql_queries/get_calculated_fqs_total_without_group.sql"),
            [],
        )
    }

    /// Returns a list of all expenses ordered by position.
    pub fn expense_list(&self) -> Vec<Expense> {
        self.fetch_rows(include_str!("sql_queries/expense_list.sql"), [])
    }

    /// Deletes a section from the database.
    ///
    /// Only succeeds if the section has no associated expense instances.
    pub fn delete_section(&self, uid: &str) {
        let count: i32 =
            self.fetch_one_value(include_str!("sql_queries/delete_section/count.sql"), &[uid]);
        if count > 0 {
            return;
        }

        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        tx.execute(
            include_str!("sql_queries/delete_section/expense_section.sql"),
            &[uid],
        )
        .expect("Échec de l'ajout de la requête à la transaction");

        tx.execute(
            include_str!("sql_queries/delete_section/section.sql"),
            &[uid],
        )
        .expect("Échec de l'ajout de la requête à la transaction");

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Deletes a QF category from the database.
    pub fn delete_fq(&self, uid: &str) {
        self.execute_write_sql(include_str!("sql_queries/delete_fq.sql"), &[uid]);
    }

    /// Updates section information in the database.
    pub fn update_section(
        &self,
        uid: &str,
        title: &str,
        color: &str,
        members_count: i32,
        adults_count: i32,
    ) {
        let existing_sections: Vec<Section> = self.fetch_rows(
            include_str!("sql_queries/update_section/existing_sections.sql"),
            (title, uid),
        );

        if !existing_sections.is_empty() {
            return;
        }

        self.execute_write_sql(
            include_str!("sql_queries/update_section/update.sql"),
            (title, color, members_count.abs(), adults_count.abs(), uid),
        );
    }

    /// Updates QF category information in the database.
    pub fn update_fq(
        &self,
        uid: &str,
        title: &str,
        coeff: &str,
        national_contribution: &str,
        online_commission_rate: &str,
        online_commission_fees: &str,
    ) {
        let coeff_f32: f32 = coeff
            .parse()
            .expect("Échec de l'analyse du coefficient (coeff) en f32");
        let national_contribution_f32: f32 = national_contribution
            .parse()
            .expect("Échec de l'analyse de la cotisation nationale en f32");
        let online_commission_rate_f32: f32 = online_commission_rate
            .parse()
            .expect("Échec de l'analyse du taux de commission en ligne en f32");
        let online_commission_fees_f32: f32 = online_commission_fees
            .parse()
            .expect("Échec de l'analyse des frais de commission en ligne en f32");

        let existing_fqs: Vec<Fq> = self.fetch_rows(
            include_str!("sql_queries/update_fq/existing_fqs.sql"),
            (title, uid),
        );

        if !existing_fqs.is_empty() {
            return;
        }

        self.execute_write_sql(
            include_str!("sql_queries/update_fq/update.sql"),
            (
                title,
                coeff_f32,
                national_contribution_f32,
                online_commission_rate_f32,
                online_commission_fees_f32,
                uid,
            ),
        );
    }

    /// Updates the display order of sections.
    pub fn update_section_order(&self, section_list: Vec<&str>) {
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        for (index, uid) in section_list.iter().enumerate() {
            tx.execute(
                include_str!("sql_queries/update_section_order.sql"),
                (index, uid),
            )
            .expect("Échec de l'ajout de la requête à la transaction");
        }

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Updates the display order of QF categories.
    pub fn update_fq_order(&self, fq_list: Vec<&str>) {
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        for (index, uid) in fq_list.iter().enumerate() {
            tx.execute(
                include_str!("sql_queries/update_fq_order.sql"),
                (index, uid),
            )
            .expect("Échec de l'ajout de la requête à la transaction");
        }

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Updates the members count for a section.
    pub fn update_members_count(&self, uid: &str, members_count: i32) {
        self.execute_write_sql(
            include_str!("sql_queries/update_members_count.sql"),
            (members_count.abs(), uid),
        );
    }

    /// Updates the adults count for a section.
    pub fn update_adults_count(&self, uid: &str, adults_count: i32) {
        self.execute_write_sql(
            include_str!("sql_queries/update_adults_count.sql"),
            (adults_count.abs(), uid),
        );
    }

    /// Updates the members count for a specific QF in a section.
    pub fn update_fq_section_members_count(
        &self,
        section_uid: &str,
        fq_uid: &str,
        members_count: i32,
    ) {
        self.execute_write_sql(
            include_str!("sql_queries/update_fq_section_members_count.sql"),
            (members_count.abs(), section_uid, fq_uid),
        );
    }

    /// Inserts a new expense and associates it with the provided sections.
    pub fn insert_new_expense(
        &self,
        title: &str,
        description: &str,
        rate: &str,
        unitprice: &str,
        section_list: Vec<&str>,
    ) {
        let rate_f32: f32 = rate.parse().expect("Échec de l'analyse du taux en f32");
        let unitprice_f32: f32 = unitprice
            .parse()
            .expect("Échec de l'analyse du prix unitaire en f32");
        let uid_expense = Uuid::new_v4().to_string();

        let sections_in_db = self.section_list_from_uid_vec(section_list);
        if sections_in_db.is_empty() {
            return;
        }

        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        tx.execute(
            include_str!("sql_queries/insert_new_expense/expense.sql"),
            (
                uid_expense.clone(),
                title,
                description,
                rate_f32,
                unitprice_f32,
            ),
        )
        .expect("Échec de l'ajout de la requête à la transaction");

        for section in sections_in_db {
            tx.execute(
                include_str!("sql_queries/insert_new_expense/expense_section.sql"),
                (uid_expense.clone(), section.uid),
            )
            .expect("Échec de l'ajout de la requête à la transaction");
        }

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    fn section_list_from_uid_vec(&self, section_list: Vec<&str>) -> Vec<Section> {
        let mut section_list_vec: Vec<Section> = vec![];
        for section in section_list {
            let mut sections_in_db = self.fetch_rows(
                include_str!("sql_queries/section_list_from_uid_vec.sql"),
                &[section],
            );
            if !sections_in_db.is_empty() {
                section_list_vec.push(
                    sections_in_db
                        .pop()
                        .expect("Impossible de récupérer la section"),
                );
            }
        }
        section_list_vec
    }

    /// Returns the total number of members across all QF categories for a section.
    pub fn get_members_fq_count_by_section(&self, section_uid: &str) -> i32 {
        let count: i32 = self.fetch_one_value(
            include_str!("sql_queries/get_members_fq_count_by_section.sql"),
            &[section_uid],
        );
        count
    }

    /// Returns a list of member counts across QF categories for all sections.
    pub fn get_members_fq_count_for_all_sections(&self) -> Vec<FqMembersCount> {
        let result: Vec<FqMembersCount> = self.fetch_rows(
            include_str!("sql_queries/get_members_fq_count_for_all_sections.sql"),
            [],
        );

        result
    }

    /// Updates expense template information in the database.
    pub fn update_expense(
        &self,
        uid: &str,
        title: &str,
        description: &str,
        rate: &str,
        unitprice: &str,
    ) {
        let rate_f32: f32 = rate.parse().expect("Échec de l'analyse du taux en f32");
        let unitprice_f32: f32 = unitprice
            .parse()
            .expect("Échec de l'analyse du prix unitaire en f32");

        self.execute_write_sql(
            include_str!("sql_queries/update_expense.sql"),
            (title, description, rate_f32, unitprice_f32, uid),
        );
    }

    /// Updates the display order of expense instances.
    pub fn update_expense_instance_order(&self, vec_expense_instance_list: Vec<&str>) {
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        for (index, uid) in vec_expense_instance_list.iter().enumerate() {
            tx.execute(
                include_str!("sql_queries/update_expense_instance_order.sql"),
                (index, uid),
            )
            .expect("Échec de l'ajout de la requête à la transaction");
        }

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Updates an expense instance with custom values.
    pub fn update_expense_instance(
        &self,
        uid_expense_instance: &str,
        unit_price: &str,
        number: &str,
        units: &str,
        units_adults: &str,
        rate: &str,
        comments: &str,
    ) {
        let unit_price_f32 = parse_f_or_none(unit_price);
        let units_f32 = parse_f_or_none(units);
        let units_adults_f32 = parse_f_or_none(units_adults);
        let rate_f32 = parse_f_or_none(rate);
        let comments_s = parse_s_or_none(comments);
        let number_f32 = parse_f_or_none(number);

        if number_f32.is_none() {
            return;
        }

        self.execute_write_sql(
            include_str!("sql_queries/update_expense_instance.sql"),
            (
                units_f32,
                units_adults_f32,
                unit_price_f32,
                rate_f32,
                comments_s,
                number_f32,
                uid_expense_instance,
            ),
        );
    }

    /// Deletes an expense instance from the database.
    pub fn delete_expense_instance(&self, uid_expense_instance: &str) {
        self.execute_write_sql(
            include_str!("sql_queries/delete_expense_instance.sql"),
            &[uid_expense_instance],
        );
    }

    /// Creates a copy of an existing expense instance.
    pub fn copy_expense_instance(&self, uid_expense_instance: &str) {
        self.execute_write_sql(
            include_str!("sql_queries/copy_expense_instance.sql"),
            (Uuid::new_v4().to_string(), uid_expense_instance),
        );
    }

    /// Updates associations between an expense and multiple sections.
    pub fn update_expense_section_association(&self, uid_expense: &str, section_list: Vec<&str>) {
        let sections_used_as_instances: Vec<SectionExpense> =
            self.get_section_expense_from_instances(uid_expense);
        let sections_in_db: Vec<Section> = self.section_list_from_uid_vec(section_list);
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

        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        tx.execute(
            include_str!("sql_queries/delete_expense/delete_expense_section.sql"),
            &[uid_expense],
        )
        .expect("Échec de l'ajout de la requête à la transaction");

        for section in sections_in_db {
            tx.execute(
                include_str!("sql_queries/insert_new_expense/expense_section.sql"),
                (uid_expense, section.uid),
            )
            .expect("Échec de l'ajout de la requête à la transaction");
        }

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Deletes an expense template from the database.
    ///
    /// Only succeeds if the expense has no associated instances.
    pub fn delete_expense(&self, uid: &str) {
        let count: i32 =
            self.fetch_one_value(include_str!("sql_queries/delete_expense/count.sql"), &[uid]);

        if count > 0 {
            return;
        }

        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        tx.execute(
            include_str!("sql_queries/delete_expense/delete_expense_section.sql"),
            &[uid],
        )
        .expect("Échec de l'ajout de la requête à la transaction");

        tx.execute(
            include_str!("sql_queries/delete_expense/delete_expense.sql"),
            &[uid],
        )
        .expect("Échec de l'ajout de la requête à la transaction");

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Returns a list of all section-expense template associations.
    pub fn get_section_expense(&self) -> Vec<SectionExpense> {
        self.fetch_rows(include_str!("sql_queries/get_section_expense.sql"), [])
    }

    /// Returns the sum of occurrences (number) for a specific section and expense instance.
    pub fn get_section_expense_cnt_from_instance(
        &self,
        section_uid: &str,
        expense_uid: &str,
    ) -> f32 {
        let count: f32 = self.fetch_one_value(
            include_str!("sql_queries/get_section_expense_cnt_from_instance.sql"),
            &[section_uid, expense_uid],
        );
        count
    }

    /// Returns section-expense association data for specific section and expense instances.
    pub fn get_section_expense_from_instance(
        &self,
        section_uid: &str,
        expense_uid: &str,
    ) -> Vec<SectionExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_section_expense_from_instance.sql"),
            &[section_uid, expense_uid],
        )
    }

    /// Returns section-expense association data from the association table.
    pub fn get_section_expense_from_association(
        &self,
        section_uid: &str,
        expense_uid: &str,
    ) -> Vec<SectionExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_section_expense_from_association.sql"),
            &[section_uid, expense_uid],
        )
    }

    /// Returns section-expense associations for an expense from its instances.
    pub fn get_section_expense_from_instances_wrapper(
        &self,
        expense_uid: &str,
    ) -> Vec<SectionExpense> {
        self.get_section_expense_from_instances(expense_uid)
    }

    fn get_section_expense_from_instances(&self, expense_uid: &str) -> Vec<SectionExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_section_expense_from_instances.sql"),
            &[expense_uid],
        )
    }

    /// Returns section-expense associations from all expense instances.
    pub fn get_section_expense_from_expenses_instances(&self) -> Vec<SectionExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_section_expense_from_expenses_instances.sql"),
            [],
        )
    }

    /// Returns section-expense associations for a specific section from its instances.
    pub fn get_section_expense_from_expenses_instances_and_section(
        &self,
        section_uid: &str,
    ) -> Vec<SectionExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_section_expense_from_expenses_instances_and_section.sql"),
            &[section_uid],
        )
    }

    /// Returns the member count for a specific section.
    pub fn get_members_count(&self, section_uid: &str) -> i32 {
        let members_count: i32 = self.fetch_one_value(
            include_str!("sql_queries/get_members_count.sql"),
            &[section_uid],
        );
        members_count
    }

    /// Returns the adult count for a specific section.
    pub fn get_adults_count(&self, section_uid: &str) -> i32 {
        let adults_count: i32 = self.fetch_one_value(
            include_str!("sql_queries/get_adults_count.sql"),
            &[section_uid],
        );
        adults_count
    }

    /// Returns section-expense template associations for a specific section.
    pub fn get_section_expense_from_expenses_instances_section(
        &self,
        section_uid: &str,
    ) -> Vec<SectionExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_section_expense_from_expenses_instances_section.sql"),
            &[section_uid],
        )
    }

    /// Returns all calculated expenses for a specific section.
    pub fn get_calculated_expenses(&self, section_uid: &str) -> Vec<CalculatedExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_calculated_expenses.sql"),
            &[section_uid],
        )
    }

    /// Returns the total sum of expenses per member for a specific section.
    pub fn get_total_per_member(&self, section_uid: &str) -> SumExpenseInstance {
        let results: Vec<SumExpenseInstance> = self.fetch_rows(
            include_str!("sql_queries/get_total_per_member.sql"),
            &[section_uid],
        );
        self.sum_expense_instance_from_vec(results)
    }

    /// Returns the sum of calculated expenses for a specific section.
    pub fn get_sum_calculated_expenses(&self, section_uid: &str) -> SumExpenseInstance {
        let results: Vec<SumExpenseInstance> = self.fetch_rows(
            include_str!("sql_queries/get_sum_calculated_expenses.sql"),
            &[section_uid],
        );
        self.sum_expense_instance_from_vec(results)
    }

    /// Returns the total sum of calculated expenses for the entire group.
    pub fn get_group_sum_calculated_expenses(&self) -> SumExpenseInstance {
        let results: Vec<SumExpenseInstance> = self.fetch_rows(
            include_str!("sql_queries/get_group_sum_calculated_expenses.sql"),
            [],
        );
        self.sum_expense_instance_from_vec(results)
    }

    /// Returns the sum of calculated expenses only for group-level instances.
    pub fn get_group_only_sum_calculated_expenses(&self) -> SumExpenseInstance {
        let results: Vec<SumExpenseInstance> = self.fetch_rows(
            include_str!("sql_queries/get_group_only_sum_calculated_expenses.sql"),
            [],
        );
        self.sum_expense_instance_from_vec(results)
    }

    fn sum_expense_instance_from_vec(&self, vec: Vec<SumExpenseInstance>) -> SumExpenseInstance {
        if let Some(item) = vec.into_iter().next() {
            item
        } else {
            SumExpenseInstance {
                sum_unit: 0.0,
                sum_total: 0.0,
            }
        }
    }

    /// Returns all calculated expenses associated with the entire group.
    pub fn get_group_calculated_expenses(&self) -> Vec<CalculatedExpense> {
        self.fetch_rows(
            include_str!("sql_queries/get_group_calculated_expenses.sql"),
            [],
        )
    }

    /// Adds a new expense instance for a section.
    pub fn add_expense_instance(&self, section_uid: &str, expense_id: &str) {
        self.execute_write_sql(
            include_str!("sql_queries/add_expense_instance.sql"),
            (Uuid::new_v4().to_string(), section_uid, expense_id),
        );
    }

    /// Updates the display order of expense templates.
    pub fn update_expense_order(&self, expense_list: Vec<&str>) {
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let mut connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_mut()
            .expect("La connexion à la base de données n'est pas initialisée");

        let tx = conn
            .transaction()
            .expect("Impossible de créer une transaction");

        for (index, uid) in expense_list.iter().enumerate() {
            tx.execute(
                include_str!("sql_queries/update_expense_order.sql"),
                (index, uid),
            )
            .expect("Échec de l'ajout de la requête à la transaction");
        }

        tx.commit()
            .expect("Échec de la validation (commit) de la transaction");
    }

    /// Utility function to execute a write SQL statement.
    pub fn execute_write_sql<T: rusqlite::Params>(&self, sql: &str, params: T) {
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.clear();

        let connection_lock = self
            .connection
            .lock()
            .expect("Impossible de verrouiller la connexion");
        let conn = connection_lock
            .as_ref()
            .expect("La connexion à la base de données n'est pas initialisée");
        let mut statement = conn
            .prepare_cached(sql)
            .expect("Impossible de préparer l'instruction SQL");
        statement
            .execute(params)
            .expect("Impossible d'exécuter l'écriture SQL");
    }

    /// Utility function to execute a read SQL query and map results to a vector.
    pub fn fetch_rows<T, P>(&self, sql: &str, params: P) -> Vec<T>
    where
        T: for<'a> TryFrom<&'a Row<'a>, Error = rusqlite::Error> + Serialize + DeserializeOwned,
        P: rusqlite::Params + Serialize,
    {
        let cache_key = generate_key(String::from(sql), &params);
        let cache_ref = self.cache.read().expect("Impossible d'accéder au cache");

        let item = cache_ref.get(&cache_key);
        if item.is_some() {
            let json_result = item.unwrap();
            let result: Vec<T> =
                serde_json::from_str(json_result).expect("Échec de désérialisation");
            return result;
        }

        //free the lock
        drop(cache_ref);

        //don't use Mutex for now, parallel connections works fine and provides data more quickly
        let conn = Connection::open(self.get_file_path())
            .expect("Impossible d'ouvrir le fichier de base de données");

        let data_iter: Vec<T> = conn
            .prepare_cached(sql)
            .expect("Impossible de préparer la requête SQL")
            .query_map(params, |row| row.try_into())
            .expect("Impossible d'exécuter un mapping vers la struct voulue")
            .flatten()
            .collect();

        let data_iter_json = serde_json::to_string(&data_iter).expect("Erreur de sérialisation");
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.insert(cache_key, data_iter_json);

        data_iter
    }

    pub fn fetch_one_value<T, P>(&self, sql: &str, params: P) -> T
    where
        T: FromSql + Serialize + for<'a> Deserialize<'a>,
        P: rusqlite::Params + Serialize,
    {
        let cache_key = generate_key(String::from(sql), &params);
        let cache_ref = self.cache.read().expect("Impossible d'accéder au cache");

        let item = cache_ref.get(&cache_key);
        if item.is_some() {
            let string_result = item.unwrap();
            let result: T = serde_json::from_str(string_result).expect("Erreur de deserialization");
            return result;
        }

        //free the lock
        drop(cache_ref);

        //don't use Mutex for now, parallel connections works fine and provides data more quickly
        let conn = Connection::open(self.get_file_path())
            .expect("Impossible d'ouvrir le fichier de base de données");

        let result = conn
            .query_row(sql, params, |row| row.get(0))
            .expect("Impossible d'obtenir une seule valeur");

        let result_json = serde_json::to_string(&result).expect("Erreur de sérialisation");
        let mut cache_ref = self.cache.write().expect("Erreur de verrouillage de cache");
        cache_ref.insert(cache_key, result_json);

        result
    }

    /// Initializes the database schema by executing migrations.
    pub fn execute_migrations(conn: &Connection) {
        conn.execute_batch(include_str!("sql_queries/_init_or_update_db.sql"))
            .expect("Impossible d'exécuter les migrations");
    }
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

pub fn generate_key<P>(sql: String, params: &P) -> String
where
    P: rusqlite::Params + Serialize,
{
    let sql_hash = generate_md5(&sql);
    let params_hash = generate_md5(params);

    format!("{}-{}", sql_hash, params_hash)
}

fn generate_md5<T>(data: &T) -> String
where
    T: Serialize,
{
    let bytes = serde_json::to_vec(data).unwrap();
    let digest = md5::compute(bytes);
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    fn test_parse_f_or_none() {
        assert_eq!(parse_f_or_none("  1.5  "), Some(1.5));
        assert_eq!(parse_f_or_none("abc"), None);
    }

    #[test]
    fn test_parse_i_or_none() {
        assert_eq!(parse_i_or_none("  10  "), Some(10));
        assert_eq!(parse_i_or_none("  10.7  "), Some(10));
        assert_eq!(parse_i_or_none("abc"), None);
    }

    #[test]
    #[serial]
    fn test_db_workflow() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_budget.lb");
        let db_path_str = db_path.to_str().unwrap();

        let repo = Repository::new();
        // Initialize DB
        repo.set_file_path(db_path_str, true);

        // Insert a section
        repo.insert_new_section("Test Section", "#FF0000", 10, 5);

        let sections = repo.section_list();
        // Find our test section
        let test_section = sections
            .iter()
            .find(|s| s.title == "Test Section")
            .expect("Section de test non trouvée");
        assert_eq!(test_section.color, "#FF0000");
        assert_eq!(sections.len(), 2);

        // Clean up
        if db_path.exists() {
            std::fs::remove_file(db_path).unwrap();
        }
    }

    #[test]
    #[serial]
    fn test_expense_workflow() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_expense.lb");
        let db_path_str = db_path.to_str().unwrap();

        let repo = Repository::new();
        repo.set_file_path(db_path_str, true);

        // Insert section and expense
        repo.insert_new_section("Section A", "#00FF00", 2, 1);
        let sections = repo.section_list();
        let section_a = sections
            .iter()
            .find(|s| s.title == "Section A")
            .expect("Section A non trouvée");

        repo.insert_new_expense("Bread", "Bakery", "100", "1.5", vec![&section_a.uid]);

        let expenses = repo.expense_list();
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].title, "Bread");

        // Clean up
        if db_path.exists() {
            std::fs::remove_file(db_path).unwrap();
        }
    }

    #[test]
    #[serial]
    fn test_fq_workflow() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_fq.lb");
        let db_path_str = db_path.to_str().unwrap();

        let repo = Repository::new();
        repo.set_file_path(db_path_str, true);

        // Insert FQ
        repo.insert_new_fq("Test FQ", "1.5", "10.0", "5.0", "2.5");

        let fqs = repo.fq_list();
        let test_fq = fqs
            .iter()
            .find(|f| f.title == "Test FQ")
            .expect("Quotient Familial (QF) de test non trouvé");
        assert_eq!(test_fq.coeff, 1.5);
        assert_eq!(fqs.len(), 1);

        // Update FQ
        repo.update_fq(&test_fq.uid, "Updated FQ", "2.0", "15.0", "6.0", "3.0");

        let fqs_updated = repo.fq_list();
        let test_fq_updated = fqs_updated
            .iter()
            .find(|f| f.title == "Updated FQ")
            .expect("Quotient Familial (QF) mis à jour non trouvé");
        assert_eq!(test_fq_updated.coeff, 2.0);

        // Duplicate prevention
        repo.insert_new_fq(
            "Updated FQ", // Same title
            "1.5",
            "10.0",
            "5.0",
            "2.5",
        );
        let fqs_after_duplicate = repo.fq_list();
        assert_eq!(fqs_after_duplicate.len(), 1); // Should not insert duplicate

        // Delete FQ
        repo.delete_fq(&test_fq_updated.uid);
        let fqs_deleted = repo.fq_list();
        assert_eq!(fqs_deleted.len(), 0);

        // Clean up
        if db_path.exists() {
            std::fs::remove_file(db_path).unwrap();
        }
    }
}
