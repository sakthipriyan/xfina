use wasm_bindgen::prelude::*;
use financial_extract_ibkr::parse_ibkr_csv;

#[wasm_bindgen]
pub fn parse_ibkr(csv_content: &str) -> Result<String, JsValue> {
    match parse_ibkr_csv(csv_content) {
        Ok(portfolio) => {
            serde_json::to_string(&portfolio)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        },
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

use financial_extract_cams::parse_cams_pdf;

#[wasm_bindgen]
pub fn parse_cams(bytes: &[u8], password: Option<String>) -> Result<String, JsValue> {
    match parse_cams_pdf(bytes, password.as_deref()) {
        Ok(portfolio) => {
            serde_json::to_string(&portfolio)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        },
        Err(e) => Err(JsValue::from_str(&e)),
    }
}
