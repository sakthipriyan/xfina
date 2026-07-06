use finx_models::BankAccountStatement;

pub fn parse_bob_bank_statement(bytes: &[u8]) -> Result<BankAccountStatement, String> {
    Ok(BankAccountStatement {
        bank_name: "Bank of Baroda".to_string(),
        ..Default::default()
    })
}
