use xfina_models::deposit::DepositAccount;

mod parser;
pub use parser::parse_hdfc_xls;

pub fn parse_hdfc_bank_statement(bytes: &[u8], _password: Option<&str>) -> Result<DepositAccount, String> {
    parse_hdfc_xls(bytes)
}
