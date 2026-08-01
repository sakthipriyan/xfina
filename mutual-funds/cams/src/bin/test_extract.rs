use pdf_extract::extract_text;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "../xfina-test-data/mutual-funds/cams/CAS_01042026-04072026_CP216237462_04072026054022992.pdf".to_string());
    let bytes = std::fs::read(&path).expect("Failed to read file");
    let password = std::env::args().nth(2).unwrap_or_else(|| "".to_string());
    let text = if password.is_empty() {
        pdf_extract::extract_text_from_mem(&bytes).unwrap()
    } else {
        pdf_extract::extract_text_from_mem_encrypted(&bytes, &password).unwrap()
    };
    println!("{}", text);
}
