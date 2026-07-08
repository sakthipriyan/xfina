use std::fs;
use xfina_ba_bob::parse_bob_xls;

#[test]
fn test_bob_parser() {
    let raw_dir = "../../../xfina-test-data/bank-accounts/bob/raw";
    let expected_dir = "../../../xfina-test-data/bank-accounts/bob/expected";

    fs::create_dir_all(expected_dir).unwrap();

    let paths = fs::read_dir(raw_dir).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extension == "xls" {
                let bytes = fs::read(&path).unwrap();
                let parsed = parse_bob_xls(&bytes).expect("Failed to parse BoB XLS");
                let json = serde_json::to_string_pretty(&parsed).unwrap();

                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let expected_path = format!("{}/{}.json", expected_dir, file_name);
                fs::write(expected_path, json).unwrap();
            }
        }
    }
}
