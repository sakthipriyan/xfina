use serde::{Deserialize, Serialize};

pub mod credit_card;
pub use credit_card::*;

pub mod deposit;
pub use deposit::*;

pub fn parse_indian_date(input: &str) -> String {
    let input = input.trim();
    
    // Extract date and optional time component like "09:41:11"
    let mut ws_parts = input.split_whitespace();
    let date_str = ws_parts.next().unwrap_or("");
    let time_str = ws_parts.next().unwrap_or("");

    // try dd/mm/yyyy or dd-mm-yyyy or dd-Mmm-yyyy
    let parts: Vec<&str> = if date_str.contains('/') {
        date_str.split('/').collect()
    } else {
        date_str.split('-').collect()
    };
    
    if parts.len() == 3 {
        let day = parts[0].trim();
        let month_str = parts[1].trim();
        let year = parts[2].trim();
        
        let month = if month_str.chars().all(|c| c.is_ascii_digit()) {
            format!("{:02}", month_str.parse::<u32>().unwrap_or(0))
        } else {
            match month_str.to_lowercase().as_str() {
                "jan" => "01".to_string(),
                "feb" => "02".to_string(),
                "mar" => "03".to_string(),
                "apr" => "04".to_string(),
                "may" => "05".to_string(),
                "jun" => "06".to_string(),
                "jul" => "07".to_string(),
                "aug" => "08".to_string(),
                "sep" => "09".to_string(),
                "oct" => "10".to_string(),
                "nov" => "11".to_string(),
                "dec" => "12".to_string(),
                _ => month_str.to_string(),
            }
        };
        
        let formatted_day = format!("{:02}", day.parse::<u32>().unwrap_or(0));
        
        let formatted_year = if year.len() == 2 {
            let year_num = year.parse::<u32>().unwrap_or(0);
            if year_num > 50 { format!("19{:02}", year_num) } else { format!("20{:02}", year_num) }
        } else {
            year.to_string()
        };
        
        let mut iso_date = format!("{}-{}-{}", formatted_year, month, formatted_day);
        if !time_str.is_empty() {
            iso_date.push('T');
            iso_date.push_str(time_str);
        }
        return iso_date;
    }
    
    input.to_string()
}

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
