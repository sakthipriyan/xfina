use xfina_ba_sbi::parse_sbi_bank_statement;
use std::fs;
use std::path::Path;

#[test]
fn test_sbi_pdf_parser() {
    let test_dir = Path::new("../../../xfina-test-data/bank-accounts/sbi");
    let raw_dir = test_dir.join("raw");
    let expected_dir = test_dir.join("expected");

    let xfina_dir = expected_dir.join("xfina");
    let rebit_dir = expected_dir.join("rebit");
    fs::create_dir_all(&xfina_dir).unwrap();
    fs::create_dir_all(&rebit_dir).unwrap();

    let password = "22391030559"; // Hardcoded for tests based on user's input

    for entry in fs::read_dir(raw_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            let bytes = fs::read(&path).unwrap();
            let filename_str = path.file_name().and_then(|s| s.to_str());
            let statement = parse_sbi_bank_statement(&bytes, Some(password), filename_str).unwrap();

            let xfina_json = serde_json::to_string_pretty(&statement.to_xfina_json()).unwrap();
            let rebit_json = serde_json::to_string_pretty(&statement.to_rebit_json()).unwrap();

            let xfina_path = xfina_dir.join(path.with_extension("json").file_name().unwrap());
            let rebit_path = rebit_dir.join(path.with_extension("json").file_name().unwrap());
            
            if xfina_path.exists() {
                let expected_xfina = fs::read_to_string(&xfina_path).unwrap();
                let expected_rebit = fs::read_to_string(&rebit_path).unwrap();
                assert_eq!(xfina_json, expected_xfina, "Mismatch for {:?}", path.file_name().unwrap());
                assert_eq!(rebit_json, expected_rebit, "Mismatch for {:?}", path.file_name().unwrap());
            } else {
                fs::write(xfina_path, xfina_json).unwrap();
                fs::write(rebit_path, rebit_json).unwrap();
            }
        }
    }
}
