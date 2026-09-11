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

   Return `XfinaError::InvalidFormat` for anything that fails *before* a single
   account field is read — a container that will not open, a UTF-8 error, a
   missing marker. That is what tells detection "not mine, try the next one".
   Reserve `ParseError` for "this **is** my format and it is damaged". Getting
   this wrong makes a wrong guess look like a corrupt file, and
   `tests/format_rejection.rs` will fail you for it.

3. **Refuse files that are not yours**: check for a structural marker your
   institution always prints before parsing on. A parser that returns `Ok` with
   a placeholder account for arbitrary input will claim every file detection
   offers it.

4. **Map to Models**: Parse the raw data directly into the shared models in `src/models/`. These models are designed to map closely to the Sahamati AA / ReBIT specifications.

5. **Split for the shared decode**: expose `parse_decoded(&Decoded, &ParseRequest)`
   alongside your public entry point, and take your content from the `Decoded`
   rather than opening the bytes yourself. Detection probes and parses against
   one read of the file.

6. **Add a `probe`**: `pub(crate) fn probe(&Decoded) -> Claim`, next to your
   parser. Match on institution boilerplate — column headings, field labels —
   never on anything specific to an account. `Claim::reason` is a fixed reason
   code by design, so matched text can never leak into a detection result.

7. **Feature Flag**: Add your parser to `Cargo.toml` as a new feature flag and include it in the `all` feature list. The flag name is also the format's id, so pick it in the `ba-`/`cc-`/`mf-`/`is-` style.

8. **Add one row to the registry**: `src/detect/registry.rs`. The `Format`
   variant, its metadata, the parse and probe dispatch, the CLI's `--as`
   values, the ids on the wire and the web app's picker are all generated from
   that table — there is nothing to update in `src/main.rs`, the bindings or
   `web/src/App.vue`.

9. **Add a filename hint** in `src/detect/hint.rs` if the institution names its
   downloads predictably. Hints only reorder the candidates; content always
   decides, so a hint that misses costs one extra probe and a hint that fires
   wrongly costs nothing.

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
4. Update the `README.md` if applicable.
5. Submit a pull request to the `main` branch.

Add your entry to `CHANGELOG.md` under `## [Unreleased]`. CI enforces this for
any change a consumer of the crate, wheel or package can observe — it is the
record of what shipped, and writing it while the change is fresh is the point.

Two branches open at once will conflict over that block. Resolving it is
usually seconds, and it is the price of having the entry written by whoever
made the change — `prepare-release` moves the block into the release, it will
not write it for you.

## Releasing

A release is one pull request and one tag.

```bash
# on the branch that carries the release, as its last commit
cargo xtask prepare-release minor        # or patch / major / an explicit 0.5.0
```

That bumps the workspace version, drafts the changelog section from the commits
since the last tag, and commits both. Edit the draft, `git commit --amend`, and
open the pull request as usual — the release rides along with the change it
describes, so there is no separate release PR to review.

Several changes can share a release: merge the earlier branches normally and run
`prepare-release` on the last one, or on a small branch of its own
(`prepare-release minor --branch` cuts `release/vX.Y.Z` for you, for shipping
what is already on `main`).

```bash
# after it is merged
cargo xtask tag-release
```

Pushing the tag is what publishes. Because there is no separate release PR to
review, this is the only checkpoint left, so it refuses unless `main` is clean,
in sync with `origin/main`, and actually declares the version in `CHANGELOG.md`;
it warns if code landed after the release was prepared and is therefore missing
from the notes.

Bump `minor` for anything breaking or feature-shaped and `patch` for fixes —
this is 0.x, so breaking changes do not need a major.

One caution: do not merge to `main` while a release is running. The tagged docs
job and `Publish Unreleased` both push `gh-pages`, and although each retries on
a rejected push, a release that fails at the docs step has already published to
three registries.

## License

By contributing to Xfina, you agree that your contributions will be licensed under the Apache 2.0 License.
