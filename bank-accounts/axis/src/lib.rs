mod parser;

use xfina_models::deposit::DepositAccount;
pub use parser::parse_axis_xls;

pub fn parse_axis_bank_statement(bytes: &[u8], filename: Option<&str>) -> Result<DepositAccount, String> {
    parse_axis_xls(bytes, filename)
}
