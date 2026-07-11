use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use crate::deposit::{HoldingNominee, TransactionType};

// -----------------------------------------------------------------------------
// AA Standard Enums
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardType {
    MasterCard,
    Visa,
    Rupay,
    #[default]
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeChoice {
    #[default]
    Yes,
    No,
}

// -----------------------------------------------------------------------------
// Xfina Extensions
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaCreditCardAccount {
    pub institution_name: Option<String>,
    pub aan: Option<String>,
    pub generated_date: Option<DateTime<Utc>>,
    pub date_only_paths: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcXfinaSummary {
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub opening_balance: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub payment_credit: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub purchases_debits: Option<Decimal>,
    
    #[serde(default)]
    pub owner_credit_breakdown: HashMap<String, f64>,
    #[serde(default)]
    pub owner_debit_breakdown: HashMap<String, f64>,
    
    pub past_dues: Option<PastDues>,
    pub reward_points_summary: Option<RewardPointsSummary>,
    #[serde(default)]
    pub reward_programs: Vec<RewardProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PastDues {
    #[serde(with = "rust_decimal::serde::float")]
    pub overlimit: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub three_months: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub two_months: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub one_month: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RewardPointsSummary {
    pub opening_balance: i32,
    pub earned: i32,
    pub disbursed: i32,
    pub adjusted_lapsed: i32,
    pub closing_balance: i32,
    pub expiring_in_30_days: Option<i32>,
    pub expiring_in_60_days: Option<i32>,
    pub default_rewards: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardProgram {
    pub program: String,
    pub bonus_points: i32,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcXfinaTransactions {
    pub start_date_derived: Option<bool>,
    pub end_date_derived: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcXfinaTransaction {
    pub owner: Option<String>,
    pub reward_points: Option<i32>,
}

// -----------------------------------------------------------------------------
// AA Standard Structures
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreditCardAccount {
    #[serde(rename = "type")]
    pub r#type: String, // "credit_card"
    pub version: f32, // typically 1.1
    pub masked_acc_number: String,
    pub linked_acc_ref: Option<String>,
    
    pub profile: Option<CcProfile>,
    pub summary: Option<CcSummary>,
    pub transactions: Option<CcTransactions>,
    
    // Xfina Extension
    pub xfina: Option<XfinaCreditCardAccount>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcProfile {
    pub holders: CcHolders,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcHolders {
    pub holder: Vec<CcHolder>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcHolder {
    pub name: String,
    pub dob: Option<NaiveDate>,
    pub mobile: Option<String>,
    pub nominee: Option<HoldingNominee>,
    pub landline: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub ckyc_compliance: Option<bool>,
    
    pub cards: Option<CcCards>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcCards {
    pub card: Vec<CcCard>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcCard {
    pub card_type: CardType,
    pub primary: TypeChoice,
    pub issued_date: Option<NaiveDate>,
    pub masked_card_number: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcSummary {
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub current_due: Option<Decimal>,
    pub last_statement_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub previous_due_amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub total_due_amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub min_due_amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub credit_limit: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub cash_limit: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub available_credit: Option<Decimal>,
    pub loyalty_points: Option<i32>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub finance_charges: Option<Decimal>,
    
    // Xfina Extension
    pub xfina: Option<CcXfinaSummary>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcTransactions {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub transaction: Vec<CcTransaction>,
    
    // Xfina Extension
    pub xfina: Option<CcXfinaTransactions>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CcTransaction {
    pub txn_id: Option<String>,
    pub txn_type: TransactionType,
    pub txn_date: Option<NaiveDate>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub value_date: Option<NaiveDate>,
    pub narration: String,
    pub statement_date: Option<NaiveDate>,
    pub mcc: Option<String>,
    pub masked_card_number: Option<String>,
    
    // Xfina Extension
    pub xfina: Option<CcXfinaTransaction>,
}
