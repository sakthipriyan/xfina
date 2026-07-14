use std::fs;
use xfina_mf_cams::parse_cams_pdf;

#[test]
fn test_cams_parser() {
    let cams_dir = "../../../xfina-test-data/mutual-funds/cams";
    
    let expected_dir = format!("{}/expected", cams_dir);
    let xfina_dir = format!("{}/xfina", expected_dir);
    let rebit_dir = format!("{}/rebit", expected_dir);
    let _ = fs::create_dir_all(&xfina_dir);
    let _ = fs::create_dir_all(&rebit_dir);

    let paths = fs::read_dir(format!("{}/raw", cams_dir)).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extension == "pdf" {
                let bytes = fs::read(&path).unwrap();
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let parsed = parse_cams_pdf(&bytes, Some("mylife@123")).expect("Failed to parse CAMS PDF");

                let xfina_json = serde_json::to_string_pretty(&parsed.to_xfina_json()).unwrap();
                let rebit_json = serde_json::to_string_pretty(&parsed.to_rebit_json()).unwrap();

                let expected_xfina_path = format!("{}/{}.json", xfina_dir, file_name);
                let expected_rebit_path = format!("{}/{}.json", rebit_dir, file_name);

                let update_expected = std::env::var("UPDATE_EXPECTED").unwrap_or_else(|_| "1".to_string());
                if update_expected == "1" {
                    fs::write(&expected_xfina_path, &xfina_json).unwrap();
                    fs::write(&expected_rebit_path, &rebit_json).unwrap();
                } else {
                    let expected_xfina = fs::read_to_string(&expected_xfina_path).unwrap();
                    assert_eq!(expected_xfina, xfina_json, "Xfina JSON mismatch for {}", file_name);
                }
            }
        }
    }
}
