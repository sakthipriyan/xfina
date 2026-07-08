use xfina_models::BankAccountStatement;

mod parser;
pub use parser::parse_hdfc_xls;

pub fn parse_hdfc_bank_statement(bytes: &[u8]) -> Result<BankAccountStatement, String> {
    parse_hdfc_xls(bytes)
}

