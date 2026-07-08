use wasm_bindgen::prelude::*;
use xfina_intl_stocks_ibkr::parse_ibkr_csv;

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

use xfina_mf_cams::parse_cams_pdf;

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

use xfina_cc_hdfc::parse_hdfc_statement;

#[wasm_bindgen]
pub fn parse_hdfc_cc(csv_content: &str) -> Result<String, JsValue> {
    match parse_hdfc_statement(csv_content) {
        Ok(stmt) => {
            serde_json::to_string(&stmt)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        },
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

use xfina_cc_icici::parse_icici_statement;

#[wasm_bindgen]
pub fn parse_icici_cc(bytes: &[u8]) -> Result<String, JsValue> {
    match parse_icici_statement(bytes) {
        Ok(stmt) => {
            serde_json::to_string(&stmt)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        },
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

use xfina_ba_hdfc::parse_hdfc_bank_statement;
#[wasm_bindgen]
pub fn parse_hdfc_ba(bytes: &[u8]) -> Result<String, JsValue> {
    match parse_hdfc_bank_statement(bytes) {
        Ok(stmt) => serde_json::to_string(&stmt).map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

use xfina_ba_icici::parse_icici_bank_statement;
#[wasm_bindgen]
pub fn parse_icici_ba(bytes: &[u8], filename: Option<String>) -> Result<String, JsValue> {
    match parse_icici_bank_statement(bytes, filename.as_deref()) {
        Ok(stmt) => serde_json::to_string(&stmt).map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

use xfina_ba_sbi::parse_sbi_bank_statement;
#[wasm_bindgen]
pub fn parse_sbi_ba(bytes: &[u8], password: Option<String>) -> Result<String, JsValue> {
    match parse_sbi_bank_statement(bytes, password.as_deref()) {
        Ok(stmt) => serde_json::to_string(&stmt).map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

use xfina_ba_bob::parse_bob_bank_statement;
#[wasm_bindgen]
pub fn parse_bob_ba(bytes: &[u8]) -> Result<String, JsValue> {
    match parse_bob_bank_statement(bytes) {
        Ok(stmt) => serde_json::to_string(&stmt).map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e))),
        Err(e) => Err(JsValue::from_str(&e)),
    }
}
