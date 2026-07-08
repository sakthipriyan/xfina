use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldershipType {
    #[default]
    Single,
    Joint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NomineeStatus {
    Registered,
    #[default]
    #[serde(rename = "NOT-REGISTERED")]
    NotRegistered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    Active,
    Inactive,
    Dormant,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Savings,
    Current,
    Overdraft,
    CashCredit,
    Nre,
    Nro,
    Fcnr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    #[default]
    Credit,
    Debit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionMode {
    Cash,
    Atm,
    Card,
    Upi,
    Imps,
    Neft,
    Rtgs,
    Cheque,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FiType {
    Deposit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Facility {
    Od,
    Cc,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositPending {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<TransactionType>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccount {
    #[serde(rename = "type")]
    pub r#type: FiType, // "deposit"
    pub masked_acc_number: String,
    pub version: f32, // typically 1.1
    pub linked_acc_ref: Option<String>,
    pub profile: Option<Profile>,
    pub summary: Option<DepositSummary>,
    pub transactions: Option<DepositTransactions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xfina: Option<XfinaDepositAccount>,
}

impl Default for DepositAccount {
    fn default() -> Self {
        Self {
            r#type: FiType::Deposit,
            masked_acc_number: String::new(),
            version: 1.1,
            linked_acc_ref: None,
            profile: None,
            summary: None,
            transactions: None,
            xfina: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaDepositAccount {
    pub institution_name: Option<String>,
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
    #[serde(rename = "type")]
    pub r#type: HoldershipType, // e.g., "SINGLE", "JOINT"
    pub holder: Vec<Holder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Holder {
    pub name: String,
    pub dob: Option<NaiveDate>,
    pub mobile: Option<String>,
    pub nominee: NomineeStatus, // "REGISTERED" or "NOT-REGISTERED"
    pub landline: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub ckyc_compliance: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xfina: Option<XfinaHolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaHolder {
    pub nominee_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositSummary {
    #[serde(with = "rust_decimal::serde::str")]
    pub current_balance: Decimal, // According to ReBIT this is String
    pub currency: Option<String>,
    pub exchge_rate: Option<String>,
    pub balance_date_time: Option<DateTime<Utc>>,
    pub opening_date: Option<NaiveDate>,
    pub status: Option<AccountStatus>,
    pub account_type: Option<AccountType>,
    pub branch: Option<String>,
    pub facility: Option<Facility>,
    pub ifsc_code: Option<String>,
    pub micr_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "rust_decimal::serde::str_option")]
    pub current_od_limit: Option<Decimal>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "rust_decimal::serde::str_option")]
    pub drawing_limit: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<DepositPending>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xfina: Option<XfinaSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaSummary {
    pub opening_balance: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositTransactions {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub transaction: Vec<DepositTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositTransaction {
    pub txn_id: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub transaction_timestamp: Option<DateTime<Utc>>,
    pub value_date: Option<NaiveDate>,
    #[serde(rename = "type")]
    pub r#type: TransactionType, // "CREDIT" or "DEBIT"
    pub mode: Option<TransactionMode>, // e.g. "UPI", "CARD", "ATM", "CASH"
    pub narration: String,
    pub reference: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    pub current_balance: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xfina: Option<XfinaTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaTransaction {
    // Fields beyond ReBIT
    pub posting_date: Option<NaiveDate>, // Since ReBIT uses DateTime for timestamp but we only have Date
}

impl DepositAccount {
    pub fn total_debits(&self) -> Decimal {
        self.transactions
            .as_ref()
            .map(|txns| {
                txns.transaction
                    .iter()
                    .filter(|t| t.r#type == TransactionType::Debit)
                    .map(|t| t.amount)
                    .sum()
            })
            .unwrap_or_else(|| Decimal::from(0))
    }

    pub fn total_credits(&self) -> Decimal {
        self.transactions
            .as_ref()
            .map(|txns| {
                txns.transaction
                    .iter()
                    .filter(|t| t.r#type == TransactionType::Credit)
                    .map(|t| t.amount)
                    .sum()
            })
            .unwrap_or_else(|| Decimal::from(0))
    }
    
    pub fn verify_running_balance(&self) -> Result<(), String> {
        let summary = self.summary.as_ref().ok_or("No summary available")?;
        let transactions = self.transactions.as_ref().ok_or("No transactions available")?;
        let mut expected_balance = summary.xfina.as_ref().and_then(|x| x.opening_balance).unwrap_or(Decimal::from(0));
        
        for txn in &transactions.transaction {
            if txn.r#type == TransactionType::Credit {
                expected_balance += txn.amount;
            } else {
                expected_balance -= txn.amount;
            }
            if expected_balance != txn.current_balance {
                // Ignore small floating errors? No, we use exact Decimal now!
                return Err(format!("Balance mismatch after txn {}: expected {}, got {}", txn.narration, expected_balance, txn.current_balance));
            }
        }
        
        if expected_balance != summary.current_balance {
            return Err(format!("Final balance mismatch: expected {}, got {}", expected_balance, summary.current_balance));
        }
        
        Ok(())
    }
}
