use rusqlite::Error;
use rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Represents a section or unit in the budget.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    /// Unique identifier for the section.
    pub uid: String,
    /// Title of the section.
    pub title: String,
    /// Hex color code for the section.
    pub color: String,
    /// Total number of members (children/teens) in the section.
    pub members_count: f32,
    /// Total number of adults/chefs in the section.
    pub adults_count: f32,
}

/// Represents an expense template.
#[derive(Debug, Serialize, Deserialize)]
pub struct Expense {
    /// Unique identifier for the expense.
    pub uid: String,
    /// Title of the expense.
    pub title: String,
    /// Detailed description of the expense.
    pub description: String,
    /// Default rate applied to the expense (percentage).
    pub rate: f32,
    /// Default unit price for the expense.
    pub unit_price: f32,
    /// Display position of the expense.
    pub position: i32,
}

/// Represents an association between a section and an expense.
#[derive(Debug, Serialize, Deserialize)]
pub struct SectionExpense {
    /// UID of the associated section.
    pub uid_section: String,
    /// UID of the associated expense.
    pub uid_expense: String,
    /// Title of the section.
    pub title_section: String,
    /// Title of the expense.
    pub title_expense: String,
    /// Number of instances of this expense for this section.
    pub count: i32,
    /// Description of the expense.
    pub description: Option<String>,
}

/// Represents a calculated expense for reporting purposes.
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatedExpense {
    /// UID of the specific expense instance.
    pub uid_expense_instance: Option<String>,
    /// UID of the section.
    pub uid_section: Option<String>,
    /// UID of the expense template.
    pub uid_expense: Option<String>,
    /// Title of the section.
    pub title_section: Option<String>,
    /// Title of the expense.
    pub title_expense: Option<String>,
    /// Comments on the expense instance.
    pub comments: Option<String>,
    /// Hex color code of the section.
    pub section_color: Option<String>,
    /// Default number of members for the expense.
    pub expenses_units: Option<f32>,
    /// Default number of adults for the expense.
    pub expenses_units_adults: Option<f32>,
    /// Default unit price.
    pub expenses_unit_price: Option<f32>,
    /// Default rate.
    pub expenses_rate: Option<f32>,
    /// Number of occurrences of this instance.
    pub expenses_instances_number: Option<f32>,
    /// Custom number of members for this instance.
    pub expenses_instances_units: Option<f32>,
    /// Custom number of adults for this instance.
    pub expenses_instances_units_adults: Option<f32>,
    /// Custom unit price for this instance.
    pub expenses_instances_unit_price: Option<f32>,
    /// Custom rate for this instance.
    pub expenses_instances_rate: Option<f32>,
    /// Actual units used (custom or default).
    pub live_units: Option<f32>,
    /// Actual adult units used (custom or default).
    pub live_units_adults: Option<f32>,
    /// Actual unit price used (custom or default).
    pub live_unit_price: Option<f32>,
    /// Actual rate used (custom or default).
    pub live_rate: Option<f32>,
    /// Rate remaining for the group (100 - live_rate).
    pub group_rate: Option<f32>,
    /// Calculated price applied to the section.
    pub applyed_price: Option<f32>,
    /// Total price applied to the section (including all units and occurrences).
    pub total_applyed_price: Option<f32>,
    /// Initial total price before any rate reduction.
    pub total_inital_price: Option<f32>,
    /// Total price applied to the group.
    pub group_applyed_total_price: Option<f32>,
    /// Unit price applied to the group per member.
    pub group_applyed_unit_price: Option<f32>,
    /// Total number of members in the entire group.
    pub group_members_count: Option<f32>,
    /// Description of the expense template.
    pub expenses_description: Option<String>,
}

/// Represents the sum of expenses for an instance or section.
#[derive(Debug, Serialize, Deserialize)]
pub struct SumExpenseInstance {
    /// Sum of unit prices.
    pub sum_unit: f32,
    /// Sum of total prices.
    pub sum_total: f32,
}

//TODO Ajouter taux pour frais de commision en ligne
//TODO Ajouter montant fixe pour frais de commision en ligne
// Actuellement =0,4+(0,8%*G90)
/// Represents a Quotient Familial (QF) category.
#[derive(Debug, Serialize, Deserialize)]
pub struct Fq {
    /// Unique identifier for the QF.
    pub uid: String,
    /// Title of the QF category.
    pub title: String,
    /// Multiplier coefficient for this QF.
    pub coeff: f32,
    /// National contribution amount for this QF.
    pub national_contribution: f32,
    /// Online commission rate (percentage).
    pub online_commission_rate: f32,
    /// Fixed fees for online commission.
    pub online_commission_fees: f32,
}

/// Represents the association between a section and a QF category.
#[derive(Debug, Serialize, Deserialize)]
pub struct FqSection {
    /// UID of the QF.
    pub uid_fq: String,
    /// UID of the section.
    pub uid_section: String,
    /// Multiplier coefficient.
    pub coeff: f32,
    /// Number of members in this section belonging to this QF.
    pub members_count: f32,
    /// Title of the section.
    pub title_section: String,
    /// Title of the QF category.
    pub title_fq: String,
}

/// Represents calculated total values for a QF category in a section.
#[derive(Debug, Serialize, Deserialize)]
pub struct FqTotal {
    /// Title of the section.
    pub title_section: String,
    /// Title of the QF category.
    pub title_fq: String,
    /// UID of the QF category.
    pub uid_fq: String,
    /// UID of the section.
    pub uid_section: String,
    /// Average weighted unit price for the section.
    pub declared_unit_price: f32,
    /// Average weighted unit price for the group.
    pub declared_group_unit_price: f32,
    /// Multiplier coefficient.
    pub coeff: f32,
    /// Calculated unit price with coefficient for the section.
    pub calculated_unit_price_with_coeff: f32,
    /// Calculated unit price with coefficient for the group.
    pub group_calculated_unit_price: f32,
    /// Total price per member for both section and group.
    pub total_group_member_price: f32,
    /// National contribution amount.
    pub national_contribution: f32,
    /// Total price per member including national contribution.
    pub total_member_price: f32,
    /// Calculated online commission fees.
    pub national_commission: f32,
    /// Final total price for the member.
    pub total: f32,
    /// Number of members declared in this QF category for the section.
    pub members_declared_count: f32,
    /// Hex color code of the section.
    pub color: String,
}

/// Represents the total national fees across all categories.
#[derive(Debug, Serialize, Deserialize)]
pub struct NationalFees {
    /// Total sum of national contributions.
    pub total_national_contribution: f32,
    /// Total sum of online commission fees.
    pub total_national_commission: f32,
}

/// Represents the number of members associated with a section in the QF context.
#[derive(Debug, Serialize, Deserialize)]
pub struct FqMembersCount {
    /// UID of the section.
    pub uid_section: String,
    /// Number of members.
    pub count: i32,
}

impl TryFrom<&Row<'_>> for Section {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid: value.get(0)?,
            title: value.get(1)?,
            color: value.get(2)?,
            members_count: value.get(3)?,
            adults_count: value.get(4)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for Fq {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid: value.get(0)?,
            title: value.get(1)?,
            coeff: value.get(2)?,
            national_contribution: value.get(3)?,
            online_commission_rate: value.get(4)?,
            online_commission_fees: value.get(5)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for FqSection {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid_section: value.get(0)?,
            uid_fq: value.get(1)?,
            coeff: value.get(2)?,
            members_count: value.get(3)?,
            title_section: value.get(4)?,
            title_fq: value.get(5)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for FqTotal {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            title_section: value.get(0)?,
            title_fq: value.get(1)?,
            uid_fq: value.get(2)?,
            uid_section: value.get(3)?,
            declared_unit_price: value.get(4)?,
            declared_group_unit_price: value.get(5)?,
            coeff: value.get(6)?,
            calculated_unit_price_with_coeff: value.get(7)?,
            group_calculated_unit_price: value.get(8)?,
            total_group_member_price: value.get(9)?,
            national_contribution: value.get(10)?,
            total_member_price: value.get(11)?,
            national_commission: value.get(12)?,
            total: value.get(13)?,
            members_declared_count: value.get(14)?,
            color: value.get(15)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for Expense {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid: value.get(0)?,
            title: value.get(1)?,
            description: value.get(2)?,
            rate: value.get(3)?,
            unit_price: value.get(4)?,
            position: value.get(5)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for NationalFees {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            total_national_contribution: value.get(0)?,
            total_national_commission: value.get(1)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for FqMembersCount {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid_section: value.get(0)?,
            count: value.get(1)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for SectionExpense {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid_section: value.get(0)?,
            uid_expense: value.get(1)?,
            title_section: value.get(2)?,
            title_expense: value.get(3)?,
            count: value.get(4)?,
            description: value.get(5)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for CalculatedExpense {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            uid_expense_instance: value.get(0)?,
            uid_section: value.get(1)?,
            uid_expense: value.get(2)?,
            title_section: value.get(3)?,
            title_expense: value.get(4)?,
            comments: value.get(5)?,
            section_color: value.get(6)?,
            expenses_units: value.get(7)?,
            expenses_units_adults: value.get(8)?,
            expenses_unit_price: value.get(9)?,
            expenses_rate: value.get(10)?,
            expenses_instances_units: value.get(11)?,
            expenses_instances_units_adults: value.get(12)?,
            expenses_instances_unit_price: value.get(13)?,
            expenses_instances_rate: value.get(14)?,
            live_units: value.get(15)?,
            live_units_adults: value.get(16)?,
            live_unit_price: value.get(17)?,
            live_rate: value.get(18)?,
            group_rate: value.get(19)?,
            applyed_price: value.get(20)?,
            total_applyed_price: value.get(21)?,
            total_inital_price: value.get(22)?,
            group_applyed_total_price: value.get(23)?,
            group_applyed_unit_price: value.get(24)?,
            group_members_count: value.get(25)?,
            expenses_description: value.get(26)?,
            expenses_instances_number: value.get(27)?,
        };

        Ok(result)
    }
}

impl TryFrom<&Row<'_>> for SumExpenseInstance {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let result = Self {
            sum_unit: value.get(0)?,
            sum_total: value.get(1)?,
        };

        Ok(result)
    }
}
