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

// Temporary stub for CAMS
#[wasm_bindgen]
pub fn parse_cams(bytes: &[u8]) -> Result<String, JsValue> {
    let _ = bytes;
    Err(JsValue::from_str("CAMS parsing not implemented yet"))
}
