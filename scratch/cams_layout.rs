use std::fs;
use xfina::models::request::ParseRequest;

fn main() {
    let bytes = fs::read("../xfina-test-data/mutual-funds/cams/raw/CAS_01042026-07082026_CP220064825_07082026082119712.pdf").unwrap();
    let pages = xfina::mutual_funds::cams::extract_spatial_pages(&bytes, Some("mylife@123")).unwrap();
    let mut all_pages_lines = Vec::new();
    for page in pages {
        let lines = xfina::mutual_funds::layout::group_into_lines(&page, 2.0);
        all_pages_lines.push(lines);
    }
    
    for (i, page_lines) in all_pages_lines.iter().enumerate() {
        for line in page_lines {
            if line.text.to_lowercase().contains("portfolio") {
                println!("Page {}: {:?}", i+1, line.text);
            }
        }
    }
}
