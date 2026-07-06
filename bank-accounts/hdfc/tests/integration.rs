use std::fs;
use std::path::Path;
use finx_ba_hdfc::parse_hdfc_xls;

#[test]
fn test_hdfc_bank_accounts() {
    let test_data_dir = Path::new("../../../financial-extract-test-data/bank-accounts/hdfc");
    
    // If the test data repo is not checked out alongside, gracefully skip
    if !test_data_dir.exists() {
        println!("Test data directory {:?} not found. Skipping integration tests.", test_data_dir);
        return;
    }

    let raw_dir = test_data_dir.join("raw");
    let expected_dir = test_data_dir.join("expected");

    for entry in fs::read_dir(raw_dir).expect("Failed to read raw directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("xls") {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            let bytes = fs::read(&path).expect("Failed to read file");
            
            let parsed_statement = parse_hdfc_xls(&bytes).expect("Failed to parse statement");
            
            let expected_file_path = expected_dir.join(format!("{}.json", file_name));
            
            // Auto-generate expected JSON if it doesn't exist
            if !expected_file_path.exists() {
                println!("Expected JSON not found for {}. Generating...", file_name);
                let json = serde_json::to_string_pretty(&parsed_statement).expect("Failed to serialize");
                fs::write(&expected_file_path, json).expect("Failed to write expected JSON");
            } else {
                let expected_json_str = fs::read_to_string(&expected_file_path).expect("Failed to read expected JSON");
                // Compare by reserializing so format matches exactly
                let actual_json = serde_json::to_string_pretty(&parsed_statement).unwrap();
                
                assert_eq!(actual_json, expected_json_str, "Mismatch for file: {}", file_name);
            }
        }
    }
}
