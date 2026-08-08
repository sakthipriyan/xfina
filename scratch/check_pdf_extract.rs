use std::fs;

fn main() {
    let bytes = fs::read("../xfina-test-data/mutual-funds/cams/raw/cams_1.pdf").unwrap();
    let text = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    println!("{}", text);
}
