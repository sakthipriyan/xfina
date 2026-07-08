pub mod pdf_parser;
pub mod layout;
pub mod parser;

use xfina_models::BankAccountStatement;

pub fn parse_sbi_bank_statement(bytes: &[u8], password: Option<&str>) -> Result<BankAccountStatement, String> {
    parser::parse_sbi_bank_statement(bytes, password)
}
