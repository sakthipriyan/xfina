fn main() {
    let mut out = String::new();
    let mut plain_text_out = pdf_extract::PlainTextOutput::new(&mut out as &mut dyn std::fmt::Write);
}
