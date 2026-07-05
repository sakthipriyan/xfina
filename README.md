# FinX

FinX is a collection of Rust libraries for extracting structured financial data from statements and reports.

The libraries are designed to run entirely in the browser via WebAssembly (WASM), allowing users to process sensitive financial documents without uploading them to any server.

## Motivation & Vision

While there are several excellent open-source financial parsers available (such as `casparser` and `processCASpdf`), they are primarily written in Python. This limits their deployment directly into the browser without a backend server, which raises privacy concerns for users who do not want to upload highly sensitive financial documents to the cloud.

By building `finx-*` in Rust, we unlock several powerful capabilities:
1. **Privacy-first WASM Deployment**: The parser can be compiled to WebAssembly (WASM) and run *entirely* within the user's web browser. Data never leaves their device.
2. **Backend & Data Science via Python**: Using `pyo3`, the exact same Rust engine can be exposed to Python for high-performance data processing pipelines.
3. **Modularity & Extensibility**: A common data model unifies disparate financial reports into standard structures, making it trivial to add support for new institutions.

## Architecture

The project is organized as a Cargo workspace with several distinct, publishable crates:

- **`mutual-funds`**:
  - **`finx-mf-cams`**: The parsing engine for CAMS Mutual Fund Combined CAS PDFs.
- **`intl-stocks`**:
  - **`finx-intl-stocks-ibkr`**: The parsing engine for Interactive Brokers (IBKR) CSV reports.
- **`credit-cards`**: Contains parsing engines for credit card statements. Currently supports:
  - **`finx-cc-hdfc`**: Parsing engine for HDFC Bank credit card statements.
  - **`finx-cc-icici`**: Parsing engine for ICICI Bank credit card statements.
- **`finx-models`**: Contains the shared data models and types (`Portfolio`, `Asset`, `Transaction`, etc.) used by all downstream parsers.
- **`finx-wasm`**: The WebAssembly bindings for running all parsers natively in the browser.

## Current Status

- **HDFC Credit Card (`finx-cc-hdfc`)**: Complete and tested with reports, including add-on users. Note: International usage has not been tested yet.
- **ICICI Credit Card (`finx-cc-icici`)**: Single user and domestic transaction testing is done.
- **Interactive Brokers (`finx-intl-stocks-ibkr`)**: Mostly stable.
- **CAMS Mutual Funds (`finx-mf-cams`)**: Work in progress (WIP). More work is required on the PDF extraction logic.

## Future Roadmap

- [ ] Create `finx-py` crate for Python bindings.

## Usage

*This library is currently under active development.*

More instructions on how to use the individual crates will be added as they are implemented.