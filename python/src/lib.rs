use ::xfina::models::request::ParseRequest;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pythonize::pythonize;

fn to_py_dict(py: Python, json_value: serde_json::Value) -> PyResult<PyObject> {
    pythonize(py, &json_value)
        .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
}

macro_rules! create_py_binding {
    ($func_name:ident, $parser_func:path) => {
        #[pyfunction]
        #[pyo3(signature = (bytes, password=None, filename=None, modified_timestamp=None, format=None))]
        fn $func_name(py: Python, bytes: &[u8], password: Option<&str>, filename: Option<&str>, modified_timestamp: Option<i64>, format: Option<&str>) -> PyResult<PyObject> {
            let req = ParseRequest::new(bytes)
                .with_password(password)
                .with_filename(filename)
                .with_modified_timestamp(modified_timestamp);

            match $parser_func(req) {
                Ok(stmt) => {
                    let mut root = serde_json::to_value(&stmt).map_err(|e| PyValueError::new_err(e.to_string()))?;
                    let data_json = if format == Some("rebit") { stmt.data.to_rebit_json() } else { stmt.data.to_xfina_json() };
                    if let Some(obj) = root.as_object_mut() {
                        obj.insert("data".to_string(), data_json);
                    }
                    to_py_dict(py, root)
                },
                Err(e) => Err(PyValueError::new_err(e.to_string())),
            }
        }
    };
}

create_py_binding!(parse_ibkr, ::xfina::intl_stocks::ibkr::parse_ibkr_csv);
create_py_binding!(parse_cams, ::xfina::mutual_funds::cams::parse_cams_pdf);
create_py_binding!(
    parse_hdfc_cc,
    ::xfina::credit_cards::hdfc::parse_hdfc_statement
);
create_py_binding!(
    parse_icici_cc,
    ::xfina::credit_cards::icici::parse_icici_statement
);
create_py_binding!(
    parse_axis_cc,
    ::xfina::credit_cards::axis::parse_axis_statement
);
create_py_binding!(
    parse_hdfc_ba,
    ::xfina::bank_accounts::hdfc::parse_hdfc_bank_statement
);
create_py_binding!(
    parse_icici_ba,
    ::xfina::bank_accounts::icici::parse_icici_bank_statement
);
create_py_binding!(
    parse_sbi_ba,
    ::xfina::bank_accounts::sbi::parse_sbi_bank_statement
);
create_py_binding!(parse_bob_ba, ::xfina::bank_accounts::bob::parse_bob_xls);
create_py_binding!(
    parse_axis_ba,
    ::xfina::bank_accounts::axis::parse_axis_bank_statement
);

/// A Python module implemented in Rust for xfina.
#[pymodule]
fn xfina(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_ibkr, m)?)?;
    m.add_function(wrap_pyfunction!(parse_cams, m)?)?;
    m.add_function(wrap_pyfunction!(parse_hdfc_cc, m)?)?;
    m.add_function(wrap_pyfunction!(parse_icici_cc, m)?)?;
    m.add_function(wrap_pyfunction!(parse_axis_cc, m)?)?;
    m.add_function(wrap_pyfunction!(parse_hdfc_ba, m)?)?;
    m.add_function(wrap_pyfunction!(parse_icici_ba, m)?)?;
    m.add_function(wrap_pyfunction!(parse_sbi_ba, m)?)?;
    m.add_function(wrap_pyfunction!(parse_bob_ba, m)?)?;
    m.add_function(wrap_pyfunction!(parse_axis_ba, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}

#[pyfunction]
fn version() -> PyResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
