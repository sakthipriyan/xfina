use xfina_mf_cams::{parser, layout};

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "../../../xfina-test-data/mutual-funds/cams/CAS_01042026-04072026_CP216237462_04072026054022992.pdf".to_string());
    let password = std::env::args().nth(2).unwrap_or_else(|| "".to_string());
    
    let bytes = std::fs::read(&path).expect("Failed to read file");
    let pages = parser::extract_spatial_pages(&bytes, Some(&password)).unwrap();
    
    for (p, page) in pages.iter().enumerate() {
        let lines = layout::group_into_lines(page, 2.0);
        for (l, line) in lines.iter().enumerate() {
            println!("Page {}, Line {}: {}", p, l, line.text);
        }
    }
}
