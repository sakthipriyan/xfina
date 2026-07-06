mod parser;

use finx_models::BankAccountStatement;
pub use parser::parse_icici_xls;

pub fn parse_icici_bank_statement(bytes: &[u8]) -> Result<BankAccountStatement, String> {
    parse_icici_xls(bytes)
}
