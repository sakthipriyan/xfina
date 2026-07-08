pub mod pdf_parser;
pub mod layout;
pub mod parser;

use xfina_models::deposit::DepositAccount;

pub fn parse_sbi_bank_statement(bytes: &[u8], password: Option<&str>) -> Result<DepositAccount, String> {
    parser::parse_sbi_bank_statement(bytes, password)
}
