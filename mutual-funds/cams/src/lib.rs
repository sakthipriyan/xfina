pub mod parser;
pub mod layout;
pub mod cas;

use finx_models::{Portfolio, InvestorInfo, Asset};

pub fn parse_cams_pdf(bytes: &[u8], password: Option<&str>) -> Result<Portfolio, String> {
    let pages = parser::extract_spatial_pages(bytes, password)?;
    
    let mut all_pages_lines = Vec::new();
    for page in pages {
        let lines = layout::group_into_lines(&page, 2.0); // 2.0 pt tolerance
        all_pages_lines.push(lines);
    }

    cas::parse_cas_lines(all_pages_lines)
}
