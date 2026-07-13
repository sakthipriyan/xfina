use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use serde_with::skip_serializing_none;

// -----------------------------------------------------------------------------
// ReBIT Schema Enums
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShareHolderEquityType {
    CommonStock,
    PreferedStock,
    AdditionalPaidInCapital,
    ContributedSurplus,
    RetainedEarning,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquityTransactionType {
    Buy,
    Sell,
    Bonus,
    Split,
    Dividend,
    Rights,
    #[default]
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionsSymbol {
    Bse,
    Nse,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldingMode {
    Demat,
    Physical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquityCategory {
    Equity,
    EquityDerivatives,
    CurrencyDerivatives,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstrumentType {
    Options,
    Futures,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquityHoldingNominee {
    Registered,
    #[serde(rename = "NOT-REGISTERED")]
    NotRegistered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum EquityFiType {
    #[default]
    Equities,
}

// -----------------------------------------------------------------------------
// ReBIT Schema Structs
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityAccount {
    #[serde(rename = "type")]
    pub r#type: EquityFiType, // "equities"
    pub masked_acc_number: String,
    pub version: f32, // typically 1.1
    pub linked_acc_ref: Option<String>,
    pub profile: Option<EquityProfile>,
    pub summary: Option<EquitySummary>,
    pub transactions: Option<EquityTransactions>,
    
    // Xfina Extension
    pub xfina: Option<XfinaEquityAccount>,
}

impl EquityAccount {
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
pub struct EquityProfile {
    pub holders: EquityHolders,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityHolders {
    pub holder: Vec<EquityHolder>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityHolder {
    pub name: String,
    pub dob: Option<NaiveDate>,
    pub mobile: Option<String>,
    pub nominee: Option<EquityHoldingNominee>,
    pub demat_id: Option<String>,
    pub landline: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub ckyc_compliance: Option<bool>,
    
    // Xfina Extension
    pub xfina: Option<XfinaEquityHolder>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquitySummary {
    pub investment: EquityInvestment,
    #[serde(with = "rust_decimal::serde::float")]
    pub investment_value: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_value: Decimal,
    
    // Xfina Extension
    pub xfina: Option<XfinaEquitySummary>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityInvestment {
    pub holdings: EquityHoldings,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityHoldings {
    #[serde(rename = "type")]
    pub r#type: Option<HoldingMode>,
    pub holding: Vec<EquityHolding>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityHolding {
    pub issuer_name: String,
    pub isin: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub units: Decimal,
    pub investment_date_time: Option<DateTime<Utc>>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub rate: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub last_traded_price: Option<Decimal>,
    pub description: Option<String>,
    
    // Xfina Extension
    pub xfina: Option<XfinaEquityHolding>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityTransactions {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub transaction: Vec<EquityTransaction>,
    
    // Xfina Extension
    pub xfina: Option<XfinaEquityTransactions>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EquityTransaction {
    pub txn_id: String,
    pub order_id: Option<String>,
    pub trade_id: Option<String>,
    pub company_name: Option<String>,
    pub symbol: Option<String>,
    pub transaction_date_time: Option<DateTime<Utc>>,
    pub exchange: Option<TransactionsSymbol>,
    pub isin: Option<String>,
    pub equity_category: Option<EquityCategory>,
    pub instrument_type: Option<InstrumentType>,
    pub option_type: Option<OptionType>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub strike_price: Option<Decimal>,
    pub narration: Option<String>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub rate: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub total_charge: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub trade_value: Option<Decimal>,
    #[serde(rename = "type")]
    pub r#type: EquityTransactionType,
    pub share_holder_equity_type: Option<ShareHolderEquityType>,
    #[serde(with = "rust_decimal::serde::float")]
    pub units: Decimal,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub other_charges: Option<Decimal>,
}

// -----------------------------------------------------------------------------
// Xfina Custom Structs
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaEquityHolding {
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub opening_balance: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub closing_balance: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub period_invested_value: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub period_realized_value: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub period_buy_units: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub period_sell_units: Option<Decimal>,
    pub period_buy_count: Option<u32>,
    pub period_sell_count: Option<u32>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaEquityHolder {
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaEquityAccount {
    pub institution_name: Option<String>,
    pub generated_date: Option<DateTime<Utc>>,
    pub date_only_paths: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaEquitySummary {
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaEquityTransactions {
}

// -----------------------------------------------------------------------------
// Implementations
// -----------------------------------------------------------------------------

impl Default for EquityAccount {
    fn default() -> Self {
        Self {
            r#type: EquityFiType::Equities,
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
