use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub isin: Option<String>,
    pub symbol: Option<String>,
    pub category: Option<String>,
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
    pub assets: Vec<Asset>,
}
