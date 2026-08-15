use serde_json::Value;
use std::fs;
use xfina::error::XfinaError;
use xfina::mutual_funds::cams::parse_cams_pdf;

/// A passwords.json entry can be a single password or an ordered list of
/// candidates (e.g. CAMS started stamping PDFs with a new password partway
/// through our test corpus, so both the old and new password need to be
/// tried). Missing/absent entries yield no candidates.
fn password_candidates(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

#[test]
fn test_cams_parser() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping integration test in CI");
        return;
    }

    let cams_dir = "../xfina-test-data/mutual-funds/cams";

    let expected_dir = format!("{}/expected", cams_dir);
    let xfina_dir = format!("{}/xfina", expected_dir);
    let rebit_dir = format!("{}/rebit", expected_dir);
    let _ = fs::create_dir_all(&xfina_dir);
    let _ = fs::create_dir_all(&rebit_dir);

    let passwords_str = fs::read_to_string(format!("{}/passwords.json", cams_dir))
        .unwrap_or_else(|_| "{}".to_string());
    let passwords: Value = serde_json::from_str(&passwords_str).unwrap_or(Value::Null);

    let paths = fs::read_dir(format!("{}/raw", cams_dir)).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extension == "pdf" {
                let bytes = fs::read(&path).unwrap();
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let file_name_with_ext = path.file_name().unwrap().to_str().unwrap();

                // A file-specific entry replaces the default candidates entirely;
                // otherwise fall back to trying every default candidate in order.
                let file_candidates = password_candidates(passwords.get(file_name_with_ext));
                let candidates = if !file_candidates.is_empty() {
                    file_candidates
                } else {
                    password_candidates(passwords.get("default"))
                };
                // No candidates at all just means "try unencrypted".
                let attempts: Vec<Option<String>> = if candidates.is_empty() {
                    vec![None]
                } else {
                    candidates.into_iter().map(Some).collect()
                };

                let mut parsed = None;
                let mut last_err = None;
                for attempt in &attempts {
                    let req = xfina::models::request::ParseRequest::new(&bytes)
                        .with_password(attempt.as_deref())
                        .with_filename(Some(file_name));
                    match parse_cams_pdf(req) {
                        Ok(p) => {
                            parsed = Some(p);
                            break;
                        }
                        Err(e) => {
                            let is_password_issue = matches!(
                                e,
                                XfinaError::IncorrectPassword | XfinaError::PasswordRequired
                            );
                            last_err = Some(e);
                            if !is_password_issue {
                                // A real parse failure - no point trying other passwords.
                                break;
                            }
                        }
                    }
                }
                let parsed = parsed.unwrap_or_else(|| {
                    panic!(
                        "Failed to parse CAMS PDF {} after trying {} password candidate(s): {:?}",
                        file_name,
                        attempts.len(),
                        last_err
                    )
                });

                let xfina_json = serialize_result(&parsed, parsed.data.to_xfina_json()).unwrap();
                let rebit_json = serialize_result(&parsed, parsed.data.to_rebit_json()).unwrap();

                let expected_xfina_path = format!("{}/{}.json", xfina_dir, file_name);
                let expected_rebit_path = format!("{}/{}.json", rebit_dir, file_name);

                let update_expected =
                    std::env::var("UPDATE_EXPECTED").unwrap_or_else(|_| "0".to_string());
                if update_expected == "1" {
                    fs::write(&expected_xfina_path, &xfina_json).unwrap();
                    fs::write(&expected_rebit_path, &rebit_json).unwrap();
                } else {
                    let expected_xfina = fs::read_to_string(&expected_xfina_path).unwrap();
                    assert_eq!(
                        expected_xfina, xfina_json,
                        "Xfina JSON mismatch for {}",
                        file_name
                    );
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
