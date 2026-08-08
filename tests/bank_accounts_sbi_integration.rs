use std::fs;
use std::path::Path;
use xfina::bank_accounts::sbi::parse_sbi_bank_statement;

#[test]
fn test_sbi_pdf_parser() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping integration test in CI");
        return;
    }

    let test_dir = Path::new("../xfina-test-data/bank-accounts/sbi");
    let raw_dir = test_dir.join("raw");
    let expected_dir = test_dir.join("expected");

    let xfina_dir = expected_dir.join("xfina");
    let rebit_dir = expected_dir.join("rebit");
    fs::create_dir_all(&xfina_dir).unwrap();
    fs::create_dir_all(&rebit_dir).unwrap();

    let passwords_str =
        fs::read_to_string(test_dir.join("passwords.json")).unwrap_or_else(|_| "{}".to_string());
    let passwords: std::collections::HashMap<String, String> =
        serde_json::from_str(&passwords_str).unwrap_or_default();

    for entry in fs::read_dir(raw_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            let bytes = fs::read(&path).unwrap();
            let filename_str = path.file_name().and_then(|s| s.to_str());
            let password = filename_str
                .and_then(|name| passwords.get(name))
                .or_else(|| passwords.get("default"))
                .map(|s| s.as_str());
            let statement = parse_sbi_bank_statement(
                xfina::models::request::ParseRequest::new(&bytes)
                    .with_password(password)
                    .with_filename(filename_str),
            )
            .unwrap();

            let xfina_json = serialize_result(&statement, statement.data.to_xfina_json()).unwrap();
            let rebit_json = serialize_result(&statement, statement.data.to_rebit_json()).unwrap();

            let xfina_path = xfina_dir.join(path.with_extension("json").file_name().unwrap());
            let rebit_path = rebit_dir.join(path.with_extension("json").file_name().unwrap());

            let update_expected =
                std::env::var("UPDATE_EXPECTED").unwrap_or_else(|_| "0".to_string());
            if update_expected == "1" {
                fs::write(&xfina_path, &xfina_json).unwrap();
                fs::write(&rebit_path, &rebit_json).unwrap();
            } else if xfina_path.exists() {
                let expected_xfina = fs::read_to_string(&xfina_path).unwrap();
                let expected_rebit = fs::read_to_string(&rebit_path).unwrap();
                assert_eq!(
                    xfina_json,
                    expected_xfina,
                    "Mismatch for {:?}",
                    path.file_name().unwrap()
                );
                assert_eq!(
                    rebit_json,
                    expected_rebit,
                    "Mismatch for {:?}",
                    path.file_name().unwrap()
                );
            } else {
                panic!("Snapshot not found. Run with UPDATE_EXPECTED=1");
            }
        }
    }
}

fn serialize_result<T: serde::Serialize>(
    stmt: &xfina::models::validation::ParseResult<T>,
    data_json: serde_json::Value,
) -> Result<String, serde_json::Error> {
    let mut root = serde_json::to_value(stmt)?;
    if let Some(obj) = root.as_object_mut() {
        obj.insert("data".to_string(), data_json);
    }
    serde_json::to_string_pretty(&root)
}
