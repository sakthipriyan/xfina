use xfina_ba_sbi::parse_sbi_bank_statement;
use std::fs;
use std::path::Path;

#[test]
fn test_sbi_pdf_parser() {
    let test_dir = Path::new("../../../xfina-test-data/bank-accounts/sbi");
    let raw_dir = test_dir.join("raw");
    let expected_dir = test_dir.join("expected");

    fs::create_dir_all(&expected_dir).unwrap();

    let password = "22391030559"; // Hardcoded for tests based on user's input

    for entry in fs::read_dir(raw_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            let bytes = fs::read(&path).unwrap();
            let filename_str = path.file_name().and_then(|s| s.to_str());
            let statement = parse_sbi_bank_statement(&bytes, Some(password), filename_str).unwrap();

            let json = serde_json::to_string_pretty(&statement).unwrap();

            let expected_path = expected_dir.join(path.with_extension("json").file_name().unwrap());
            
            // If expected file exists, assert it matches. Otherwise, write it (initial creation).
            if expected_path.exists() {
                let expected_json = fs::read_to_string(&expected_path).unwrap();
                assert_eq!(json, expected_json, "Mismatch for {:?}", path.file_name().unwrap());
            } else {
                fs::write(expected_path, json).unwrap();
            }
        }
    }
}
