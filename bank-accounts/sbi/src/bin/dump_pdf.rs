use xfina_ba_sbi::{pdf_parser, layout};

fn main() {
    let bytes = std::fs::read("../../../xfina-test-data/bank-accounts/sbi/raw/AccountStatement_05072026_211225.pdf").unwrap();
    let pages = pdf_parser::extract_spatial_pages(&bytes, Some("22391030559")).unwrap();
    
    for (page_idx, page) in pages.iter().enumerate() {
        println!("--- PAGE {} ---", page_idx + 1);
        let lines = layout::group_into_lines(page, 2.0); // 2.0 pt tolerance
        for (i, line) in lines.iter().enumerate() {
            let min_x = line.chars.first().map(|c| c.x0).unwrap_or(0.0);
            println!("{:03}: [{:06.1}, {:06.1}] {}", i, min_x, line.baseline, line.text);
        }
    }
}
