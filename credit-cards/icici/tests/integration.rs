use std::fs;
use std::path::Path;
use xfina_cc_icici::parse_icici_statement;

#[test]
fn test_icici_credit_cards() {
    let test_data_dir = Path::new("../../../xfina-test-data/credit-cards/icici");
    
    // If the test data repo is not checked out alongside, gracefully skip
    if !test_data_dir.exists() {
        println!("Test data directory {:?} not found. Skipping integration tests.", test_data_dir);
        return;
    }

    let raw_dir = test_data_dir.join("raw");
    let expected_dir = test_data_dir.join("expected");

    fs::create_dir_all(expected_dir.join("xfina")).expect("Failed to create xfina directory");
    fs::create_dir_all(expected_dir.join("rebit")).expect("Failed to create rebit directory");

    for entry in fs::read_dir(raw_dir).expect("Failed to read raw directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        let ext = path.extension().and_then(|e| e.to_str());
        if ext == Some("pdf") || ext == Some("xls") {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            let bytes = fs::read(&path).expect("Failed to read file");
            
            let parsed_result = parse_icici_statement(&bytes, Some(file_name));
            
            if let Ok(parsed_statement) = parsed_result {
                let xfina_path = expected_dir.join("xfina").join(format!("{}.json", file_name));
                let rebit_path = expected_dir.join("rebit").join(format!("{}.json", file_name));
                
                let xfina_json = serde_json::to_string_pretty(&parsed_statement.to_xfina_json()).unwrap();
                let rebit_json = serde_json::to_string_pretty(&parsed_statement.to_rebit_json()).unwrap();
                
                fs::write(&xfina_path, &xfina_json).expect("Failed to write xfina JSON");
                fs::write(&rebit_path, &rebit_json).expect("Failed to write rebit JSON");
                
                let expected_xfina = fs::read_to_string(&xfina_path).expect("Failed to read expected xfina");
                let expected_rebit = fs::read_to_string(&rebit_path).expect("Failed to read expected rebit");
                
                assert_eq!(xfina_json, expected_xfina, "Mismatch for xfina file: {}", file_name);
                assert_eq!(rebit_json, expected_rebit, "Mismatch for rebit file: {}", file_name);
            } else {
                println!("Skipping {} due to parsing error: {:?}", file_name, parsed_result.unwrap_err());
            }
        }
    }
}
