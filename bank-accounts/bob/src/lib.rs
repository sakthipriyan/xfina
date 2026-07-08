use xfina_models::BankAccountStatement;

pub fn parse_bob_bank_statement(_bytes: &[u8]) -> Result<BankAccountStatement, String> {
    let mut stmt = BankAccountStatement::default();
    stmt.statement.institution_name = Some("Bank of Baroda".to_string());
    Ok(stmt)
}
