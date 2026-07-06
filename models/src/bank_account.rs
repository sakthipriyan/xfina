use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    pub date: String,
    pub value_date: Option<String>,
    pub description: String,
    pub reference_number: Option<String>,
    pub tx_type: String, // "Credit" or "Debit"
    pub amount: f64,
    pub balance: Option<f64>,
}

use crate::credit_card::CustomerInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BankAccountStatement {
    pub bank_name: String,
    pub account_number: Option<String>,
    pub customer_info: CustomerInfo,
    pub statement_start_date: Option<String>,
    pub statement_end_date: Option<String>,
    pub opening_balance: Option<f64>,
    pub closing_balance: Option<f64>,
    pub transactions: Vec<BankTransaction>,
}
