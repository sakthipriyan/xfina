use financial_extract_models::Portfolio;

pub fn parse_cams_pdf(bytes: &[u8], password: Option<&str>) -> Result<Portfolio, String> {
    // TODO: implement logic using pdf-extract to read text
    // and parse CAMS mutual fund transactions.
    Ok(Portfolio {
        assets: vec![],
    })
}
