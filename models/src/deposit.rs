use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use serde_with::skip_serializing_none;

// -----------------------------------------------------------------------------
// ReBIT Schema Enums
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Savings,
    Current,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldingNominee {
    Registered,
    #[serde(rename = "NOT-REGISTERED")]
    NotRegistered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldersType {
    #[default]
    Single,
    Joint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SummaryFacility {
    Od,
    Cc,
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
pub enum StatusTypes {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionMode {
    Cash,
    Atm,
    Card,
    Upi,
    Ft,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum FiType {
    #[default]
    Deposit,
}

// -----------------------------------------------------------------------------
// ReBIT Schema Structs
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccount {
    #[serde(rename = "type")]
    pub r#type: FiType, // "deposit"
    pub masked_acc_number: String,
    pub version: f32, // typically 1.1
    pub linked_acc_ref: Option<String>,
    pub profile: Option<Profile>,
    pub summary: Option<Summary>,
    pub transactions: Option<Transactions>,
    
    // Xfina Extension
    pub xfina: Option<XfinaDepositAccount>,
}

impl DepositAccount {
    pub fn to_xfina_json(&self) -> serde_json::Value {
        let mut val = serde_json::to_value(self).unwrap();
        crate::serializer::transform_to_xfina(&mut val);
        val
    }

    pub fn to_rebit_json(&self) -> serde_json::Value {
        let mut val = serde_json::to_value(self).unwrap();
        let paths = self.xfina.as_ref()
            .and_then(|x| x.date_only_paths.clone())
            .unwrap_or_default();
        crate::serializer::transform_to_rebit(&mut val, &paths, "".to_string());
        val
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub holders: Holders,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub pending: Option<Pending>,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_balance: Decimal,
    pub currency: Option<String>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    #[serde(rename = "exchgeRate")]
    pub exchange_rate: Option<Decimal>,
    pub balance_date_time: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub r#type: Option<AccountType>,
    pub branch: Option<String>,
    pub facility: Option<SummaryFacility>,
    pub ifsc_code: Option<String>,
    pub micr_code: Option<String>,
    pub opening_date: Option<NaiveDate>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub current_od_limit: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub drawing_limit: Option<Decimal>,
    pub status: Option<StatusTypes>,
    
    // Xfina Extension
    pub xfina: Option<XfinaSummary>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Pending {
    pub transaction_type: Option<TransactionType>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Holders {
    #[serde(rename = "type")]
    pub r#type: HoldersType, // e.g., "SINGLE", "JOINT"
    pub holder: Vec<Holder>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Holder {
    pub name: String,
    pub dob: Option<NaiveDate>,
    pub mobile: Option<String>,
    pub nominee: Option<HoldingNominee>, // "REGISTERED" or "NOT-REGISTERED"
    pub landline: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub ckyc_compliance: Option<bool>,
    
    // Xfina Extension
    pub xfina: Option<XfinaHolder>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub transaction: Vec<Transaction>,
    
    // Xfina Extension
    pub xfina: Option<XfinaTransactions>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(rename = "type")]
    pub r#type: TransactionType, // "DEBIT" or "CREDIT"
    pub mode: Option<TransactionMode>, // e.g., "CASH", "UPI"
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_balance: Decimal,
    pub transaction_timestamp: Option<DateTime<Utc>>,
    pub value_date: Option<NaiveDate>,
    pub txn_id: Option<String>,
    pub narration: String,
    pub reference: Option<String>,
}

// -----------------------------------------------------------------------------
// Xfina Custom Structs
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaHolder {
    pub customer_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaDepositAccount {
    pub institution_name: Option<String>,
    pub generated_date: Option<DateTime<Utc>>,
    pub date_only_paths: Option<Vec<String>>,
}


#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaSummary {
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub opening_balance: Option<Decimal>,
    pub account_product: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaTransactions {
}

// -----------------------------------------------------------------------------
// Implementations
// -----------------------------------------------------------------------------

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
                return Err(format!("Balance mismatch after txn {}: expected {}, got {}", txn.narration, expected_balance, txn.current_balance));
            }
        }
        
        if expected_balance != summary.current_balance {
            return Err(format!("Final balance mismatch: expected {}, got {}", expected_balance, summary.current_balance));
        }
        
        Ok(())
    }
}
