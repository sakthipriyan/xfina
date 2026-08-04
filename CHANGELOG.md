# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-08-04

### Added
- **Deployment:** Re-architected website deployment pipeline via `cargo xtask deploy-site` to generate immutable, permanent archives for all minor releases in `gh-pages`.
- **Infrastructure:** Updated `.github/workflows/publish.yml` to trigger orchestrated GitHub Actions deployments and handle safe concurrency locking.

### Changed
- **Web App:** Configured Vite with `base: './'` for path-agnostic artifact generation.
- **Web App:** Transitioned from fetching version lists via GitHub API to a dynamically generated `versions.json` registry.

### Fixed
- **Web App:** Fixed Issue #23 where the version dropdown failed to properly display or navigate to the latest active version.

## [0.1.2] - 2026-08-03

## [0.1.1] - 2026-08-03

### Added
- **Parsers:** Added a new `format` parameter (`"rebit"` or `"xfina"`) to all WASM, Python, and CLI parsers to allow toggling between strict ReBIT AA schema compliance and extended Xfina schemas.
- **Python:** Fully implemented PyO3 bindings for all statement parsers in the `xfina` PyPI package (the previous release accidentally omitted them).
- **CLI:** Added the `--format` option to the CLI tool.

### Fixed
- **CI/CD:** Upgraded NPM in GitHub Actions to v11+ to fully support passwordless OIDC Trusted Publishing, fixing the `ENEEDAUTH` error.
- **Documentation:** Updated all READMEs (Rust, WASM, Python) with correct function signatures, new format parameters, and accurate code examples.
## [0.1.0] - 2026-08-03

### Added
- **Parsers:** Support for parsing PDF/XLS statements from major Indian financial institutions:
  - Bank Accounts: HDFC, ICICI, SBI, Bank of Baroda, Axis Bank
  - Credit Cards: HDFC, ICICI
  - Mutual Funds: CAMS (CAS)
  - International Stocks: Interactive Brokers (IBKR)
- **Data Models:** Centralized `xfina-models` package standardizing financial schema based on RBI Account Aggregator specifications.
- **CLI Tool:** Unified `xfina-cli` binary for parsing statements directly from the terminal and exporting to JSON.
- **Language Bindings:** 
  - `python`: Python bindings published to PyPI using PyO3/Maturin.
  - `wasm`: WebAssembly module (`xfina-wasm`) published to NPM using `wasm-pack` for browser integration.
- **Web App:** Vue 3 + Tailwind CSS frontend interface demonstrating local, privacy-preserving WASM parsing.
- **CI/CD:** Automated GitHub Actions pipeline for testing and publishing to Crates.io, NPM, PyPI, and GitHub Pages.
