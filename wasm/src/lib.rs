use wasm_bindgen::prelude::*;
use xfina::models::request::ParseRequest;
use xfina::models::validation::ParseResult;

fn serialize_result<T: serde::Serialize>(
    stmt: &ParseResult<T>,
    data_json: serde_json::Value,
) -> Result<String, JsValue> {
    let mut root = serde_json::to_value(stmt).map_err(|e| JsValue::from_str(&e.to_string()))?;
    if let Some(obj) = root.as_object_mut() {
        obj.insert("data".to_string(), data_json);
    }
    serde_json::to_string(&root)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

macro_rules! create_wasm_binding {
    ($func_name:ident, $parser_func:path) => {
        #[wasm_bindgen]
        pub fn $func_name(
            bytes: &[u8],
            password: Option<String>,
            filename: Option<String>,
            modified_timestamp: Option<i64>,
            format: Option<String>,
        ) -> Result<String, JsValue> {
            let req = ParseRequest::new(bytes)
                .with_password(password.as_deref())
                .with_filename(filename.as_deref())
                .with_modified_timestamp(modified_timestamp);

            match $parser_func(req) {
                Ok(stmt) => {
                    let data_json = if format.as_deref() == Some("rebit") {
                        stmt.data.to_rebit_json()
                    } else {
                        stmt.data.to_xfina_json()
                    };
                    serialize_result(&stmt, data_json)
                }
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        }
    };
}

create_wasm_binding!(parse_ibkr, xfina::intl_stocks::ibkr::parse_ibkr_csv);
create_wasm_binding!(parse_cams, xfina::mutual_funds::cams::parse_cams_pdf);
create_wasm_binding!(
    parse_hdfc_cc,
    xfina::credit_cards::hdfc::parse_hdfc_statement
);
create_wasm_binding!(
    parse_icici_cc,
    xfina::credit_cards::icici::parse_icici_statement
);
create_wasm_binding!(
    parse_axis_cc,
    xfina::credit_cards::axis::parse_axis_statement
);
create_wasm_binding!(
    parse_hdfc_ba,
    xfina::bank_accounts::hdfc::parse_hdfc_bank_statement
);
create_wasm_binding!(
    parse_icici_ba,
    xfina::bank_accounts::icici::parse_icici_bank_statement
);
create_wasm_binding!(
    parse_sbi_ba,
    xfina::bank_accounts::sbi::parse_sbi_bank_statement
);
create_wasm_binding!(parse_bob_ba, xfina::bank_accounts::bob::parse_bob_xls);
create_wasm_binding!(
    parse_axis_ba,
    xfina::bank_accounts::axis::parse_axis_bank_statement
);
