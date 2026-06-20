use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvestorInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub contact: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub isin: Option<String>,
    pub symbol: Option<String>,
    pub category: Option<String>,
    pub total_units: f64,
    pub invested_value: f64,
    pub current_nav: Option<f64>,
    pub current_nav_date: Option<String>,
    pub current_value: Option<f64>,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub date: String,
    pub tx_type: String,
    pub amount: f64,
    pub units: f64,
    pub nav: Option<f64>,
    pub balance: Option<f64>,
    pub fee: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub investor_info: InvestorInfo,
    pub generated_date: Option<String>,
    pub assets: Vec<Asset>,
}
