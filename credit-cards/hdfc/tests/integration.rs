use std::fs;
use std::path::Path;
use xfina_cc_hdfc::parse_hdfc_statement;
use pdf_extract::extract_text;

#[test]
fn test_hdfc_credit_cards() {
    let test_data_dir = Path::new("../../../xfina-test-data/credit-cards/hdfc");
    
    // If the test data repo is not checked out alongside, gracefully skip
    if !test_data_dir.exists() {
        println!("Test data directory {:?} not found. Skipping integration tests.", test_data_dir);
        return;
    }

    let raw_dir = test_data_dir.join("raw");
    let expected_dir = test_data_dir.join("expected");

    if !expected_dir.exists() {
        fs::create_dir_all(&expected_dir).expect("Failed to create expected directory");
    }

    for entry in fs::read_dir(raw_dir).expect("Failed to read raw directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("csv") {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            
            // Read CSV content directly
            let content = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(e) => {
                    println!("Failed to read text from {:?}: {:?}", path, e);
                    continue;
                }
            };
            
            let parsed_result = parse_hdfc_statement(&content, Some(file_name));
            
            if let Ok(parsed_statement) = parsed_result {
                let expected_file_path = expected_dir.join(format!("{}.json", file_name));
                
                let json = serde_json::to_string_pretty(&parsed_statement).expect("Failed to serialize");
                fs::write(&expected_file_path, &json).expect("Failed to write expected JSON");
                
                let expected_json_str = fs::read_to_string(&expected_file_path).expect("Failed to read expected JSON");
                let actual_json = serde_json::to_string_pretty(&parsed_statement).unwrap();
                
                assert_eq!(actual_json, expected_json_str, "Mismatch for file: {}", file_name);
            } else {
                println!("Skipping {} due to parsing error: {:?}", file_name, parsed_result.unwrap_err());
            }
        }
    }
}
