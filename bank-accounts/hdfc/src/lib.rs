use finx_models::BankAccountStatement;

mod parser;
pub use parser::parse_hdfc_xls;

pub fn parse_hdfc_bank_statement(bytes: &[u8]) -> Result<BankAccountStatement, String> {
    parse_hdfc_xls(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hdfc_xls() {
        let bytes = std::fs::read("test_data/Acct_Statement_XXXXXXXX2144_05072026.xls").unwrap();
        let stmt = parse_hdfc_xls(&bytes).unwrap();
        println!("{:?}", stmt);
    }
}
