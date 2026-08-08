# Xfina (Python Bindings)

[![Crates.io](https://img.shields.io/crates/v/xfina.svg?color=orange)](https://crates.io/crates/xfina)
[![PyPI](https://img.shields.io/pypi/v/xfina.svg?color=blue)](https://pypi.org/project/xfina/)
[![npm](https://img.shields.io/npm/v/xfina-wasm.svg?color=yellow)](https://www.npmjs.com/package/xfina-wasm)

**Xfina** is a blazingly fast library for parsing Indian financial statements (Bank Accounts, Credit Cards, Mutual Funds) and international brokers (IBKR), written in Rust and exposed to Python via PyO3.

It converts raw PDFs, Excel files, and CSVs into structured, ReBIT-compliant JSON dictionaries in milliseconds.

## Installation

```bash
pip install xfina
```

## Quick Start

The Python bindings expose the exact same parsing functions as the Rust core. Since it is backed by Rust, it requires raw bytes or string contents to be passed in, rather than file paths.

### Parsing a Bank Statement (Excel)

```python
import xfina
import os

filename = "hdfc_statement.xls"
modified_ts = int(os.path.getmtime(filename))

# Read the file as raw bytes
with open(filename, "rb") as f:
    file_bytes = f.read()

# Parse it! Providing the filename and modified timestamp helps improve parsing accuracy
result = xfina.parse_hdfc_ba(
    file_bytes, 
    password=None, 
    filename=filename,
    modified_timestamp=modified_ts,
    format="xfina"
)

# The returned dictionary contains both the validation report and the financial data
print(f"Validation Status: {result['validation']['overall']}")
print(f"Account Name: {result['data']['profile']['holders']['holder'][0]['name']}")
```

### Parsing a CAMS Mutual Fund Statement (PDF)

```python
import xfina
import json

with open("cams_cas.pdf", "rb") as f:
    file_bytes = f.read()

# Pass the password to decrypt the PDF
mf_data = xfina.parse_cams(file_bytes, password="PAN1234567")

# It returns a standard Python dictionary. You can easily dump it to JSON:
with open("output.json", "w") as f:
    json.dump(mf_data, f, indent=2)
```

### Parsing an IBKR Statement (CSV)

```python
import xfina

# All parsers now uniformly expect raw bytes, even for CSVs
with open("ibkr_activity.csv", "rb") as f:
    file_bytes = f.read()

result = xfina.parse_ibkr(file_bytes, format="rebit")
```

## Available Parsers

All parsers return a structured Python dictionary that mirrors the ReBIT JSON schema. Each parser accepts an optional `format` parameter which defaults to `"xfina"`, but can be `"rebit"` for strict AA schema compliance without our extended data fields.

| Category | Institution | Format | Python Function | Input Type |
|---|---|---|---|---|
| Bank Account | HDFC | `.xls` | `parse_hdfc_ba(bytes, **kwargs)` | `bytes` |
| Bank Account | ICICI | `.xls` | `parse_icici_ba(bytes, **kwargs)` | `bytes` |
| Bank Account | SBI | PDF | `parse_sbi_ba(bytes, **kwargs)` | `bytes` |
| Bank Account | BOB | `.xls` | `parse_bob_ba(bytes, **kwargs)` | `bytes` |
| Bank Account | Axis | `.xls` | `parse_axis_ba(bytes, **kwargs)` | `bytes` |
| Credit Card | HDFC | CSV | `parse_hdfc_cc(bytes, **kwargs)` | `bytes` |
| Credit Card | ICICI | `.xls` | `parse_icici_cc(bytes, **kwargs)` | `bytes` |
| Mutual Funds | CAMS | PDF | `parse_cams(bytes, **kwargs)` | `bytes` |
| Intl Stocks | IBKR | CSV | `parse_ibkr(bytes, **kwargs)` | `bytes` |

*Note: All Python functions accept the following optional keyword arguments: `password`, `filename`, `modified_timestamp`, and `format`.*
