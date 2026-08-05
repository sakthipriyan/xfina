use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pythonize::pythonize;
use ::xfina::intl_stocks::ibkr::parse_ibkr_csv;
use ::xfina::mutual_funds::cams::parse_cams_pdf;
use ::xfina::credit_cards::hdfc::parse_hdfc_statement;
use ::xfina::credit_cards::icici::parse_icici_statement;
use ::xfina::bank_accounts::hdfc::parse_hdfc_bank_statement;
use ::xfina::bank_accounts::icici::parse_icici_bank_statement;
use ::xfina::bank_accounts::sbi::parse_sbi_bank_statement;
use ::xfina::bank_accounts::bob::parse_bob_xls;
use ::xfina::bank_accounts::axis::parse_axis_bank_statement;

fn to_py_dict(py: Python, json_value: serde_json::Value) -> PyResult<PyObject> {
    pythonize(py, &json_value)
        .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
}

#[pyfunction]
#[pyo3(signature = (csv_content, format=None))]
fn parse_ibkr(py: Python, csv_content: &str, format: Option<&str>) -> PyResult<PyObject> {
    match parse_ibkr_csv(csv_content) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, password=None, format=None, filename=None))]
fn parse_cams(py: Python, bytes: &[u8], password: Option<&str>, format: Option<&str>, filename: Option<&str>) -> PyResult<PyObject> {
    match parse_cams_pdf(bytes, password, filename) {
        Ok(portfolio) => {
            let json = if format == Some("rebit") { portfolio.to_rebit_json() } else { portfolio.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (content, filename=None, format=None))]
fn parse_hdfc_cc(py: Python, content: &str, filename: Option<&str>, format: Option<&str>) -> PyResult<PyObject> {
    match parse_hdfc_statement(content, filename) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, filename=None, format=None))]
fn parse_icici_cc(py: Python, bytes: &[u8], filename: Option<&str>, format: Option<&str>) -> PyResult<PyObject> {
    match parse_icici_statement(bytes, filename) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, password=None, format=None))]
fn parse_hdfc_ba(py: Python, bytes: &[u8], password: Option<&str>, format: Option<&str>) -> PyResult<PyObject> {
    match parse_hdfc_bank_statement(bytes, password) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, filename=None, format=None))]
fn parse_icici_ba(py: Python, bytes: &[u8], filename: Option<&str>, format: Option<&str>) -> PyResult<PyObject> {
    match parse_icici_bank_statement(bytes, filename) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, password=None, filename=None, format=None))]
fn parse_sbi_ba(py: Python, bytes: &[u8], password: Option<&str>, filename: Option<&str>, format: Option<&str>) -> PyResult<PyObject> {
    match parse_sbi_bank_statement(bytes, password, filename) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, format=None))]
fn parse_bob_ba(py: Python, bytes: &[u8], format: Option<&str>) -> PyResult<PyObject> {
    match parse_bob_xls(bytes) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

#[pyfunction]
#[pyo3(signature = (bytes, filename=None, format=None))]
fn parse_axis_ba(py: Python, bytes: &[u8], filename: Option<&str>, format: Option<&str>) -> PyResult<PyObject> {
    match parse_axis_bank_statement(bytes, filename) {
        Ok(stmt) => {
            let json = if format == Some("rebit") { stmt.to_rebit_json() } else { stmt.to_xfina_json() };
            to_py_dict(py, json)
        },
        Err(e) => Err(PyValueError::new_err(e.to_string())),
    }
}

/// A Python module implemented in Rust for xfina.
#[pymodule]
fn xfina(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_ibkr, m)?)?;
    m.add_function(wrap_pyfunction!(parse_cams, m)?)?;
    m.add_function(wrap_pyfunction!(parse_hdfc_cc, m)?)?;
    m.add_function(wrap_pyfunction!(parse_icici_cc, m)?)?;
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
