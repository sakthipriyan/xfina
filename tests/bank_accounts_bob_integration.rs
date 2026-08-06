use std::fs;
use xfina::bank_accounts::bob::parse_bob_xls;

#[test]
fn test_bob_parser() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping integration test in CI");
        return;
    }

    let raw_dir = "../xfina-test-data/bank-accounts/bob/raw";
    let expected_dir = "../xfina-test-data/bank-accounts/bob/expected";

    let xfina_dir = format!("{}/xfina", expected_dir);
    let rebit_dir = format!("{}/rebit", expected_dir);
    fs::create_dir_all(&xfina_dir).unwrap();
    fs::create_dir_all(&rebit_dir).unwrap();

    let paths = fs::read_dir(raw_dir).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extension == "xls" {
                let bytes = fs::read(&path).unwrap();
                let parsed = parse_bob_xls(xfina::models::request::ParseRequest::new(&bytes)).expect("Failed to parse BoB XLS");
                let file_name = path.file_stem().unwrap().to_str().unwrap();

                let xfina_json = serialize_result(&parsed, parsed.data.to_xfina_json()).unwrap();
                let rebit_json = serialize_result(&parsed, parsed.data.to_rebit_json()).unwrap();

                let expected_xfina_path = format!("{}/{}.json", xfina_dir, file_name);
                let expected_rebit_path = format!("{}/{}.json", rebit_dir, file_name);

                let update_expected = std::env::var("UPDATE_EXPECTED").unwrap_or_else(|_| "0".to_string());
                if update_expected == "1" {
                    fs::write(&expected_xfina_path, &xfina_json).unwrap();
                    fs::write(&expected_rebit_path, &rebit_json).unwrap();
                } else {
                    let expected_xfina = fs::read_to_string(&expected_xfina_path).unwrap();
                    let expected_rebit = fs::read_to_string(&expected_rebit_path).unwrap();
                    assert_eq!(expected_xfina, xfina_json, "Xfina JSON mismatch for {}", file_name);
                    assert_eq!(expected_rebit, rebit_json, "ReBIT JSON mismatch for {}", file_name);
                }
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
