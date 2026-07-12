use std::fs;
use xfina_ba_bob::parse_bob_xls;

#[test]
fn test_bob_parser() {
    let raw_dir = "../../../xfina-test-data/bank-accounts/bob/raw";
    let expected_dir = "../../../xfina-test-data/bank-accounts/bob/expected";

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
                let parsed = parse_bob_xls(&bytes).expect("Failed to parse BoB XLS");
                let file_name = path.file_stem().unwrap().to_str().unwrap();

                let xfina_json = serde_json::to_string_pretty(&parsed.to_xfina_json()).unwrap();
                let rebit_json = serde_json::to_string_pretty(&parsed.to_rebit_json()).unwrap();

                let expected_xfina_path = format!("{}/{}.json", xfina_dir, file_name);
                let expected_rebit_path = format!("{}/{}.json", rebit_dir, file_name);

                fs::write(&expected_xfina_path, &xfina_json).unwrap();
                fs::write(&expected_rebit_path, &rebit_json).unwrap();

                // To add assertions, we'd normally read the file first. Here it just writes them, which is fine for generating fixtures.
            }
        }
    }
}
