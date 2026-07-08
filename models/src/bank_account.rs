use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatementMetadata {
    pub institution_name: String,
    pub account_number: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub generated_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub holders: Holders,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Holders {
    pub r#type: String, // e.g., "SINGLE", "JOINT"
    pub holder: Vec<Holder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Holder {
    pub name: String,
    pub dob: Option<NaiveDate>,
    pub mobile: Option<String>,
    pub nomineename: Option<String>,
    pub landline: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub ckyc_compliance: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositSummary {
    pub current_balance: Option<f64>,
    pub currency: Option<String>,
    pub balance_date_time: Option<DateTime<Utc>>,
    pub opening_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub account_type: Option<String>,
    
    // Xfina Extended field (not in strict AA, but extracted from PDFs)
    pub opening_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositTransaction {
    pub txn_id: Option<String>,
    pub amount: f64,
    pub date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub r#type: String, // "CREDIT" or "DEBIT"
    pub narration: String, // replaces 'description'
    pub reference: Option<String>, // replaces 'reference_number'
    pub current_balance: Option<f64>, // replaces 'balance'
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountStatement {
    // 1. Universal Metadata (Fields not inside the AA inner FI JSON, but part of the outer wrapper/request)
    pub statement: StatementMetadata,
    
    // 2. Standard AA Blocks
    pub profile: Profile,
    pub summary: DepositSummary,
    pub transactions: Vec<DepositTransaction>,
}

// Implement totals as methods rather than stored fields
impl BankAccountStatement {
    pub fn total_debits(&self) -> f64 {
        self.transactions.iter()
            .filter(|t| t.r#type.eq_ignore_ascii_case("debit"))
            .map(|t| t.amount)
            .sum()
    }
    
    pub fn total_credits(&self) -> f64 {
        self.transactions.iter()
            .filter(|t| t.r#type.eq_ignore_ascii_case("credit"))
            .map(|t| t.amount)
            .sum()
    }
}

