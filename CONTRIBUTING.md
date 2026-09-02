# Contributing to Xfina

Thank you for your interest in contributing to Xfina! This project thrives on community contributions, especially for adding new parsers for various Indian financial institutions.

## Getting Started

1. **Fork the Repository**
2. **Install Rust**: Xfina is built in Rust. Install it via [rustup](https://rustup.rs/).
3. **Run the Setup**:
   ```bash
   cargo build
   cargo test
   ```

## Development Workflow

### Adding a New Parser

Xfina’s core parsing logic lives in the `xfina` crate, organized by domain:
- `bank-accounts/`
- `credit-cards/`
- `mutual-funds/`
- `intl-stocks/`

1. **Create the Parser**: Create a new module in the appropriate directory (e.g., `bank-accounts/newbank.rs`).
2. **Implement Error Handling**: All parsers must return `Result<T, crate::error::XfinaError>`. Use `?` for early returns.
3. **Map to Models**: Parse the raw data directly into the shared models in `src/models/`. These models are designed to map closely to the Sahamati AA / ReBIT specifications.
4. **Feature Flag**: Add your parser to `Cargo.toml` as a new feature flag and include it in the `all` feature list.
5. **Update Targets**: 
   - Export your parser in `src/lib.rs`
   - Add it to the CLI in `src/main.rs`
   - Add WASM bindings in `wasm/src/lib.rs`
   - Add Python bindings in `python/src/lib.rs`
   - Add UI support in `web/src/App.vue`

### Testing Requirements

Integration tests are mandatory for all parsers to ensure that future changes do not break existing snapshot outputs.

1. Create a new integration test in the `tests/` directory (e.g., `tests/bank_accounts_newbank_integration.rs`).
2. You will need sample financial statements (e.g., PDFs, Excel files, CSVs). Since these contain PII, **DO NOT commit test data to this repository.**

   This extends past the files themselves. This repository is public, so no value taken from a real statement belongs in source, comments, doc examples, `CHANGELOG.md`, commit messages, or a PR or issue description — no holder names, no account or card numbers even when masked, no real balances or amounts, no narrations, addresses or contact details. Invent values that keep the shape and drop the content, and describe a bug structurally rather than by quoting the statement that found it. `AGENTS.md` has the full rule and examples.
3. Test data is expected to reside in a sibling directory: `../xfina-test-data/`.
4. When writing your test, follow this strict snapshot pattern to ensure tests run reliably in CI and can be easily updated locally:

```rust
let update_expected = std::env::var("UPDATE_EXPECTED").unwrap_or_else(|_| "0".to_string());
if update_expected == "1" {
    // Write new snapshots
    fs::write(&expected_xfina_path, &xfina_json).unwrap();
    fs::write(&expected_rebit_path, &rebit_json).unwrap();
} else {
    // Assert against existing snapshots
    let expected_xfina = fs::read_to_string(&expected_xfina_path).unwrap();
    let expected_rebit = fs::read_to_string(&expected_rebit_path).unwrap();
    assert_eq!(expected_xfina, xfina_json, "Xfina JSON mismatch");
    assert_eq!(expected_rebit, rebit_json, "ReBIT JSON mismatch");
}
```

To update snapshots locally, run:
```bash
UPDATE_EXPECTED=1 cargo test
```

## Pull Request Process

1. Ensure your code passes all tests (`cargo test`).
2. Ensure your code is properly formatted (`cargo fmt`).
3. If modifying WASM interfaces, rebuild the WASM bundle (`cd wasm && wasm-pack build --target web`).
4. Update the `README.md` and `task.md` if applicable.
5. Submit a pull request to the `main` branch.

## License

By contributing to Xfina, you agree that your contributions will be licensed under the Apache 2.0 License.
