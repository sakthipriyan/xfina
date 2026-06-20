# financial-extract

`financial-extract` is a suite of robust, privacy-preserving Rust libraries designed for parsing financial documents, such as mutual fund Consolidated Account Statements (CAS) PDFs and broker CSV exports.

## Motivation & Vision

While there are several excellent open-source financial parsers available (such as `casparser` and `processCASpdf`), they are primarily written in Python. This limits their deployment directly into the browser without a backend server, which raises privacy concerns for users who do not want to upload highly sensitive financial documents to the cloud.

By building `financial-extract` in Rust, we unlock several powerful capabilities:
1. **Privacy-first WASM Deployment**: The parser can be compiled to WebAssembly (WASM) and run *entirely* within the user's web browser. Data never leaves their device.
2. **Backend & Data Science via Python**: Using `pyo3`, the exact same Rust engine can be exposed to Python for high-performance data processing pipelines.
3. **Modularity & Extensibility**: A common data model unifies disparate financial reports into standard structures, making it trivial to add support for new institutions.

## Architecture

The project is organized as a Cargo workspace with several distinct, publishable crates:

- **`models`**: Contains the shared data models and types (`Portfolio`, `Asset`, `Transaction`, etc.) used by all downstream parsers.
- **`cams`**: The parsing engine for CAMS Mutual Fund CAS PDFs.
- **`ibkr`**: The parsing engine for Interactive Brokers (IBKR) CSV reports.

### Future Roadmap

- [ ] Implement core parsing logic for `cams` (PDF text extraction).
- [ ] Implement core parsing logic for `ibkr` (CSV processing).
- [ ] Add parsers for NSDL and CDSL demat accounts.
- [ ] Create `financial-extract-wasm` crate for browser deployments.
- [ ] Create `financial-extract-py` crate for Python bindings.

## Usage

*This library is currently under active development.*

More instructions on how to use the individual crates will be added as they are implemented.