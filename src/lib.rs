//! # Xfina
//!
//! **Xfina** is a comprehensive suite of financial statement parsers specifically tailored for the Indian financial ecosystem (Bank Accounts, Credit Cards, Mutual Funds) and international brokers (IBKR).
//!
//! This crate acts as the **facade library** that unifies all the individual parser sub-crates under a single, ergonomic namespace. It allows you to import only what you need, while ensuring all output perfectly adheres to the standardized ReBIT JSON schemas via `xfina-models`.
//!
//! ## Example Usage
//!
//! ```no_run
//! use xfina::bank_accounts::hdfc::parse_hdfc_bank_statement;
//!
//! # fn main() -> Result<(), String> {
//! let bytes = std::fs::read("statement.xls").unwrap();
//! let account = parse_hdfc_bank_statement(&bytes, None)?;
//!
//! println!("Account Name: {}", account.profile.holders.holder[0].name);
//! # Ok(())
//! # }
//! ```
//!
//! ## Organization
//!
//! The library is organized by financial instrument category:
//!
//! - [`models`]: Shared data models (ReBIT / Sahamati AA standard compatible)
//! - [`bank_accounts`]: Parsers for savings and current accounts (HDFC, ICICI, SBI, BOB, Axis)
//! - [`credit_cards`]: Parsers for credit card statements (HDFC, ICICI)
//! - [`mutual_funds`]: Parsers for mutual fund statements (CAMS CAS)
//! - [`intl_stocks`]: Parsers for international broker statements (IBKR)

pub use xfina_models as models;

pub mod mutual_funds {
    pub use xfina_mf_cams as cams;
}

pub mod intl_stocks {
    pub use xfina_intl_stocks_ibkr as ibkr;
}

pub mod credit_cards {
    pub use xfina_cc_hdfc as hdfc;
    pub use xfina_cc_icici as icici;
}

pub mod bank_accounts {
    pub use xfina_ba_hdfc as hdfc;
    pub use xfina_ba_icici as icici;
    pub use xfina_ba_sbi as sbi;
    pub use xfina_ba_bob as bob;
    pub use xfina_ba_axis as axis;
}
