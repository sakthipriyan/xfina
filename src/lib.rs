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
//! # fn main() -> Result<(), xfina::error::XfinaError> {
//! let bytes = std::fs::read("statement.xls").unwrap();
//! let account = parse_hdfc_bank_statement(&bytes, None)?;
//!
//! // Example: Accessing parsed data (this will compile if you load an actual account)
//! if let Some(profile) = account.profile {
//!     println!("Account Name: {}", profile.holders.holder[0].name);
//! }
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



pub mod mutual_funds {
    #[cfg(feature = "mf-cams")]
    pub mod cams;
    #[cfg(feature = "mf-cams")]
    pub mod cas;
    #[cfg(feature = "mf-cams")]
    pub mod layout;
}

pub mod intl_stocks {
    #[cfg(feature = "is-ibkr")]
    pub mod ibkr;
}

pub mod credit_cards {
    #[cfg(feature = "cc-hdfc")]
    pub mod hdfc;
    #[cfg(feature = "cc-icici")]
    pub mod icici;
}

pub mod bank_accounts {
    #[cfg(feature = "ba-hdfc")]
    pub mod hdfc;
    #[cfg(feature = "ba-icici")]
    pub mod icici;
    #[cfg(feature = "ba-sbi")]
    pub mod sbi;
    #[cfg(feature = "ba-bob")]
    pub mod bob;
    #[cfg(feature = "ba-axis")]
    pub mod axis;
    
    #[cfg(feature = "ba-sbi")]
    pub(crate) mod layout;
    #[cfg(feature = "ba-sbi")]
    pub(crate) mod pdf_parser;
}

pub mod models;
pub mod error;
