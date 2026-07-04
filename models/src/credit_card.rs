use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInfo {
    pub name: String,
    pub address: String,
    pub customer_gstn: Option<String>,
}

impl Default for CustomerInfo {
    fn default() -> Self {
        CustomerInfo {
            name: String::new(),
            address: String::new(),
            customer_gstn: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountSummary {
    pub opening_balance: f64,
    pub payment_credit: f64,
    pub purchases_debits: f64,
    pub finance_charges: f64,
    pub total_dues: f64,
    #[serde(default)]
    pub owner_credit_breakdown: HashMap<String, f64>,
    #[serde(default)]
    pub owner_debit_breakdown: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PastDues {
    pub overlimit: f64,
    pub three_months: f64,
    pub two_months: f64,
    pub one_month: f64,
    pub current_dues: f64,
    pub minimum_amount_due: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardTransaction {
    pub owner: String,
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub tx_type: String, // e.g. "Debit", "Credit"
    pub reward_points: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
pub struct RewardProgram {
    pub program: String,
    pub bonus_points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreditCardStatement {
    pub card_no: Option<String>,
    pub aan: Option<String>,
    pub customer_info: CustomerInfo,
    pub payment_due_date: Option<String>,
    pub statement_date: Option<String>,
    pub statement_start_date: Option<String>,
    pub statement_end_date: Option<String>,
    pub total_amount_due: Option<f64>,
    pub minimum_amount_due: Option<f64>,
    pub credit_limit: Option<f64>,
    pub available_limit: Option<f64>,
    pub available_cash_limit: Option<f64>,
    pub account_summary: Option<AccountSummary>,
    pub past_dues: Option<PastDues>,
    pub transactions: Vec<CreditCardTransaction>,
    pub reward_points_summary: Option<RewardPointsSummary>,
    pub reward_programs: Vec<RewardProgram>,
}
