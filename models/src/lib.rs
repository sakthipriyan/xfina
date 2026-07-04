use serde::{Deserialize, Serialize};

pub mod credit_card;
pub use credit_card::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvestorInfo {
    pub account_number: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub contact: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub folio_number: Option<String>,
    pub isin: Option<String>,
    pub symbol: Option<String>,
    pub category: Option<String>,
    pub period_units: f64,
    pub period_invested_value: f64,
    pub period_realized_value: f64,

    pub total_units: f64,
    pub total_cost_basis: f64,
    pub current_nav: Option<f64>,
    pub current_nav_date: Option<String>,
    pub current_value: Option<f64>,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub date: String,
    pub tx_type: String,
    pub description: Option<String>,
    pub amount: f64,
    pub units: f64,
    pub nav: Option<f64>,
    pub balance: Option<f64>,
    pub fee: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub investor_info: InvestorInfo,
    pub statement_start_date: Option<String>,
    pub statement_end_date: Option<String>,
    pub generated_date: Option<String>,
    pub assets: Vec<Asset>,
}
