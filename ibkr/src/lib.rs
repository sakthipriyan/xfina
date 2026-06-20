use financial_extract_models::Portfolio;

pub fn parse_ibkr_csv(csv_content: &str) -> Result<Portfolio, String> {
    // TODO: implement logic using csv crate to parse IBKR trades.
    Ok(Portfolio {
        assets: vec![],
    })
}
