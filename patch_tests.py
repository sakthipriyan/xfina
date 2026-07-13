import os
import glob
import re

def process_file(path):
    with open(path, 'r') as f:
        content = f.read()

    # We need to change:
    # 1. create_dir_all(&expected_dir) to create xfina and rebit dirs
    # 2. generating and checking JSONs
    
    # Let's just do a string replacement if possible, or AST matching, but regex is easier.
    
    if "to_xfina_json" in content:
        print(f"Skipping {path}, already patched")
        return

    # Update directory creation
    content = content.replace(
        'fs::create_dir_all(&expected_dir).expect("Failed to create expected directory");',
        '''fs::create_dir_all(expected_dir.join("xfina")).expect("Failed to create xfina directory");
        fs::create_dir_all(expected_dir.join("rebit")).expect("Failed to create rebit directory");'''
    )

    # Update JSON serialization and assertion
    # The block usually looks like:
    # let expected_file_path = expected_dir.join(format!("{}.json", file_name));
    # let json = serde_json::to_string_pretty(&parsed_statement).expect("Failed to serialize");
    # fs::write(&expected_file_path, &json).expect("Failed to write expected JSON");
    # let expected_json_str = fs::read_to_string(&expected_file_path).expect("Failed to read expected JSON");
    # let actual_json = serde_json::to_string_pretty(&parsed_statement).unwrap();
    # assert_eq!(actual_json, expected_json_str, "Mismatch for file: {}", file_name);
    
    # We will just replace it entirely using regex

    pattern = re.compile(r'let expected_file_path = expected_dir\.join\(format\!\("\{}\.json", file_name\)\);[\s\S]*?assert_eq!\(actual_json, expected_json_str, "Mismatch for file: {}", file_name\);')
    
    replacement = '''
                let xfina_path = expected_dir.join("xfina").join(format!("{}.json", file_name));
                let rebit_path = expected_dir.join("rebit").join(format!("{}.json", file_name));
                
                let xfina_json = serde_json::to_string_pretty(&parsed_statement.to_xfina_json()).unwrap();
                let rebit_json = serde_json::to_string_pretty(&parsed_statement.to_rebit_json()).unwrap();
                
                fs::write(&xfina_path, &xfina_json).expect("Failed to write xfina JSON");
                fs::write(&rebit_path, &rebit_json).expect("Failed to write rebit JSON");
                
                let expected_xfina = fs::read_to_string(&xfina_path).expect("Failed to read expected xfina");
                let expected_rebit = fs::read_to_string(&rebit_path).expect("Failed to read expected rebit");
                
                assert_eq!(xfina_json, expected_xfina, "Mismatch for xfina file: {}", file_name);
                assert_eq!(rebit_json, expected_rebit, "Mismatch for rebit file: {}", file_name);'''
                
    content = pattern.sub(replacement.strip(), content)

    # If it was an xls/pdf parser, the variable might not be `parsed_statement`. Some use `parsed_result`
    # Wait, the pattern matched everything. Let's make sure it matched.
    if "xfina_json" not in content:
        print(f"Failed to match in {path}")
    else:
        with open(path, 'w') as f:
            f.write(content)
        print(f"Patched {path}")


def main():
    paths = glob.glob('*/tests/integration.rs') + glob.glob('*/*/tests/integration.rs')
    for p in paths:
        process_file(p)

if __name__ == '__main__':
    main()
