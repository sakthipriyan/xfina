mod parser;

use xfina_models::deposit::DepositAccount;
pub use parser::parse_icici_xls;

pub fn parse_icici_bank_statement(bytes: &[u8], filename: Option<&str>) -> Result<DepositAccount, String> {
    parse_icici_xls(bytes, filename)
}
