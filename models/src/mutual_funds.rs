use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// -----------------------------------------------------------------------------
// ReBIT Schema Enums
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MfHoldingType {
    Sole,
    Joint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MfTransactionType {
    Buy,
    Sell,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MfHoldingStatus {
    Individual,
    Minor,
    Huf,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MfHoldingSubtype {
    AnyOneOrSurvivor,
    EitherOrSurvivor,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MfHoldingMode {
    Demat,
    Physical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MfHoldingNominee {
    Registered,
    NotRegistered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FundType {
    Equity,
    Debt,
    Hybrid,
    SolutionOrientedSchemes,
    Others,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemePlan {
    Direct,
    Regular,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemeOption {
    Reinvest,
    Payout,
    GrowthType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemeTypes {
    EquitySchemes,
    DebtSchemes,
    HybridSchemes,
    SolutionOrientedSchemes,
    OtherSchemes,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemeCategory {
    MultiCapFund,
    LargeCapFund,
    LargeAndMidCapFund,
    MidcapFund,
    SmallCapFund,
    DividendYieldFund,
    ValueFund,
    ContraFund,
    FocusedFund,
    SectoralOrThematic,
    Elss,
    OvernightFund,
    LiquidFund,
    UltraShortDurationFund,
    LowDurationFund,
    MoneyMarketFund,
    ShortDurationFund,
    MediumDurationFund,
    MediumToLongDurationFund,
    LongDurationFund,
    DynamicBond,
    CorporateBondFund,
    CreditRiskFund,
    BankingAndPsuFund,
    GiltFund,
    GiltFundWith10YearConstantDuration,
    FloaterFund,
    ConservativeHybridFund,
    BalancedHybridFund,
    AggressiveHybridFund,
    DynamicAssetAllocationOrBalancedAdvantage,
    MultiAssetAllocation,
    ArbitrageFund,
    EquitySavings,
    RetirementFund,
    ChildrensFund,
    IndexFundsOrEtfs,
    FofsOverseasOrDomestic,
}

// -----------------------------------------------------------------------------
// ReBIT Schema Structs
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MutualFundsAccount {
    #[serde(rename = "type")]
    pub r#type: String, // fixed="mutualfunds"
    pub masked_acc_number: String,
    pub version: String,
    pub linked_acc_ref: String,
    
    pub profile: Option<MfProfile>,
    pub summary: Option<MfSummary>,
    pub transactions: Option<MfTransactions>,
    
    // Xfina Extension
    pub xfina: Option<XfinaMutualFundsAccount>,
}

impl MutualFundsAccount {
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
pub struct MfProfile {
    pub holders: MfHolders,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfHolders {
    pub r#type: Option<MfHoldingType>,
    pub holder: Vec<MfHolder>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfHolder {
    pub name: String,
    pub dob: Option<NaiveDate>,
    pub mobile: Option<String>, // Schema says xs:integer but string is safer for phone
    pub nominee: Option<MfHoldingNominee>,
    pub demat_id: Option<String>,
    pub landline: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub pan: Option<String>,
    pub ckyc_compliance: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfSummary {
    #[serde(with = "rust_decimal::serde::float")]
    pub investment_value: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_value: Decimal,
    
    pub investment: MfInvestment,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfInvestment {
    pub holdings: MfHoldings,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfHoldings {
    pub holding: Vec<MfHolding>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfHolding {
    pub amc: Option<String>,
    pub registrar: Option<String>,
    pub scheme_code: Option<String>,
    pub isin: Option<String>,
    pub ucc: Option<String>,
    pub amfi_code: Option<String>,
    pub folio_no: Option<String>,
    pub dividend_type: Option<String>,
    pub fatca_status: Option<String>,
    pub mode: Option<MfHoldingMode>,
    
    #[serde(with = "rust_decimal::serde::float")]
    pub units: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub closing_units: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub lien_units: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub rate: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub nav: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub locking_units: Decimal,

    // Xfina Extension
    pub xfina: Option<XfinaMutualFundsHolding>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfTransactions {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub transaction: Vec<MfTransaction>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfTransaction {
    pub txn_id: Option<String>,
    pub amc: Option<String>,
    pub registrar: Option<String>,
    pub scheme_code: Option<String>,
    pub scheme_plan: Option<SchemePlan>,
    pub isin: Option<String>,
    pub amfi_code: Option<String>,
    pub fund_type: Option<FundType>,
    pub scheme_option: Option<SchemeOption>,
    pub scheme_types: Option<SchemeTypes>,
    pub scheme_category: Option<SchemeCategory>,
    pub ucc: Option<String>,
    
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub closing_units: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub lien_units: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub nav: Decimal,
    
    pub nav_date: Option<NaiveDate>,
    
    #[serde(rename = "type")]
    pub r#type: Option<MfTransactionType>,
    
    pub order_date: Option<NaiveDate>,
    pub execution_date: Option<NaiveDate>,
    
    pub lock_in_flag: Option<String>,
    pub lock_in_days: Option<String>,
    
    pub mode: Option<MfHoldingMode>,
    pub narration: Option<String>,

    // Xfina Extension
    pub xfina: Option<XfinaMutualFundsTransaction>,
}


// -----------------------------------------------------------------------------
// Xfina Custom Structs
// -----------------------------------------------------------------------------

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaMutualFundsAccount {
    pub generated_date: Option<DateTime<Utc>>,
    pub date_only_paths: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaMutualFundsHolding {
    pub scheme_name: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_invested: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_value: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub unrealized_pl: Decimal,
    
    #[serde(with = "rust_decimal::serde::float")]
    pub period_buy_units: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub period_sell_units: Decimal,
    
    pub period_buy_count: u32,
    pub period_sell_count: u32,
    
    #[serde(with = "rust_decimal::serde::float")]
    pub opening_balance: Decimal,
    
    pub nav_date: Option<NaiveDate>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfinaMutualFundsTransaction {
    #[serde(with = "rust_decimal::serde::float")]
    pub units: Decimal,
    
    #[serde(with = "rust_decimal::serde::float")]
    pub fees: Decimal,
    
    pub transaction_date_time: Option<DateTime<Utc>>,
}
