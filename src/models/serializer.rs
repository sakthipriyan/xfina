use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;

pub fn naive_date_to_epoch(d: NaiveDate) -> i64 {
    d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()
}

pub fn transform_to_xfina(val: &mut Value) {
    match val {
        Value::Object(map) => {
            for v in map.values_mut() {
                transform_to_xfina(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                transform_to_xfina(v);
            }
        }
        Value::String(s) => {
            if let Ok(ndt) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                *val = Value::Number(serde_json::Number::from(naive_date_to_epoch(ndt)));
            } else if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                *val = Value::Number(serde_json::Number::from(dt.with_timezone(&Utc).timestamp()));
            }
        }
        _ => {}
    }
}

pub fn transform_to_rebit(val: &mut Value, date_only_paths: &[String], current_path: String) {
    match val {
        Value::Object(map) => {
            map.remove("xfina");
            let keys: Vec<String> = map.keys().cloned().collect();
            for k in keys {
                if let Some(v) = map.get_mut(&k) {
                    let next_path = if current_path.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", current_path, k)
                    };
                    transform_to_rebit(v, date_only_paths, next_path);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                transform_to_rebit(v, date_only_paths, current_path.clone());
            }
        }
        Value::String(s) => {
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                if date_only_paths.contains(&current_path) {
                    *val = Value::String(dt.with_timezone(&Utc).format("%Y-%m-%d").to_string());
                }
            }
        }
        _ => {}
    }
}
