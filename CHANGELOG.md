# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Web:** imported statements are listed oldest first by period, and each card shows its period.

## [0.5.0] - 2026-09-11

### Added

- **One entry point that works out what a file is.** `xfina::parse` takes bytes
  and a filename and returns the parsed account together with the format it
  detected, the category, the institution and the evidence. `detect` answers the
  same question without parsing, and `formats` lists what a build can read.
  Closes #64 and #65.
- **`xfina detect` and `xfina formats`** on the CLI.
- **`generatedDateDerived`** on all four account extensions, set when the
  statement date came from the filename or the file's modification time rather
  than the statement itself. The web UI has rendered this as an "est." badge
  since it was written; nothing had ever set it.
- `modified_timestamp` is now the last resort for a statement's generated date.
  It was accepted by every surface and read by nothing.

### Changed

- **Docs:** `versions.json` is now a single ordered `series` list — released versions newest first, the unreleased build last — instead of a list plus separate `latest` and `unreleased` keys repeating parts of it. Each entry records the `commit` its directory was built from, and the web app reads the badge from there, falling back to the hash Vite baked in for local development. A wrong commit id is now corrected in one place, without rebuilding the site. The unreleased entry is keyed `minor: "unreleased"` — the same string the version dropdown already selects by — rather than carrying a flag that would restate what its key already says.

- **Breaking — bindings.** The ten per-parser exports are gone from both the
  WASM and Python packages, replaced by `parse`, `detect`, `formats` and
  `version`. The WASM functions take an options object and return a plain
  object; failures arrive as `{ error: { kind, ... } }` rather than a thrown
  string, so a caller can tell "needs a password" from "cannot read this".
  Python raises `XfinaParseError`, carrying the same information as attributes.
- **Breaking — web.** The category and institution pickers are gone. One drop
  zone takes any number of statements at once; each is read on its own and
  appears as a card under a heading for its kind of account, saying which
  account it is and whose name is on it. Files that need a password collect in
  an "Action required" section, where each is unlocked individually or
  dropped. The supported formats are listed from `formats()` rather than
  hardcoded, so the list cannot drift from what the build actually reads --
  each with the extension the institution actually uses, whether it arrives
  locked, a link to where it is downloaded from and the trail to reach it.
- **Breaking — CLI.** `xfina parse <category> <institution> <file>` is now
  `xfina parse <FILE> [--as <format>]`, and `-f/--format` is `--schema`, which
  frees "format" to mean the parser rather than the output shape.
- **Breaking — Rust.** `ParseRequest` gains a `format` field. The ten per-format
  functions are unchanged and still public.
- Wrong-format failures return `InvalidFormat` instead of `ParseError`, so a
  file in the wrong container is distinguishable from a damaged one.
- `XfinaError` gains `kind()`, `UnrecognizedFormat` and `FormatNotEnabled`.
- The SBI PDF path reports a missing or wrong password as `PasswordRequired` /
  `IncorrectPassword` rather than a `ParseError` string.
- A statement's generated date now prefers what the institution printed over
  what the filename says. This changes SBI's precedence, which was inverted.

### Fixed

- **Web App / Docs:** The commit badge in the header linked to a commit that does not exist on every published site except `/unreleased/`. The hash was baked in by Vite from whatever `HEAD` the deploy ran on, and for the tagged builds that was a pre-squash branch commit the merge discarded — `/0.4/` and the root mirror pointed at `a8a9d75`. The tagged deploy now resolves the tag itself (`git rev-list -n 1 v<version>`), which names a commit that survives the merge, and passes it to the build.

- **Parsers no longer claim files they cannot read.** IBKR returned a populated
  account with placeholder values for arbitrary text; HDFC credit cards, SBI and
  CAMS did the same for content they did not recognise. Each now checks for its
  own structural marker first.
- Each input is decoded once however many parsers examine it. The two copies of
  the PDF spatial extractor are now one.

### Internal

- `Schema { Xfina, Rebit }` and one serializer replace twelve copies of the same
  envelope-building helper.
- `chrono` and `rust_decimal` are no longer optional; `src/models` always needed
  them, so `--no-default-features` never compiled.
- CI builds the empty feature set and each parser feature alone.

## [0.4.1] - 2026-09-02

### Fixed
- **Parsers (ICICI BA):** ICICI shuffles a day's rows relative to the balance column, so a strict running-balance check failed on rows that were not themselves wrong — an inward remittance printed after the debit it funded, for instance. Each day is now put back into the order its printed balances describe before any balance is derived from row order. Closes #53.

  Every row implies the balance it started from (`printed - delta`), which makes the day a walk that uses each row exactly once — an Eulerian path, found in O(n) with Hierholzer's algorithm rather than by trying permutations. A day is rewritten only when a walk consumes all of its rows, and rows never move across dates, so the outcome is either a day that chains exactly or the statement's own order untouched. When a day is reordered, `transactions.xfina.reordered` says so and the web app labels the transaction list.

  Across the ICICI corpus this turns 2 statements from `warning` into `passed` (2 rows moved in each) and leaves the other 4 byte-identical; in every case the set of transactions and both balances are unchanged.
- **Privacy:** Parser comments carried sample rows copied verbatim from real statements — an account number and holder name in the ICICI bank parser, an account number and balances in the Axis bank parser, and a real payment amount in the Axis credit card parser. All replaced with invented values of the same shape. This repository is public; the statements it is developed against are not.

### Added
- **Docs:** `AGENTS.md` opens with a top-priority rule against putting anything derived from a real statement into this repository or its commits, PRs and issues, with the substitutions to use instead. `CONTRIBUTING.md` points at it from the existing "do not commit test data" note.

## [0.4.0] - 2026-09-02

### Added
- **Parsers (Axis CC):** The card variant printed in the statement title (e.g. `Neo`) is now extracted into `summary.xfina.cardProduct`.
- **Web App:** Credit card statement details now show the card variant as `Product`, mirroring the bank account view.

### Fixed
- **Python wheels:** the PyPI job built a single wheel on `ubuntu-latest` with
  CPython 3.11, so only that exact combination could `pip install xfina`
  without a Rust toolchain — 0.2.4 shipped nothing but a
  `manylinux_2_34_x86_64` CPython 3.11 wheel. Fixed on three fronts, mirroring
  [sakthipriyan/xfingine](https://github.com/sakthipriyan/xfingine):
  - The extension now builds against the **stable ABI** (`pyo3/abi3-py38`), so
    one wheel per OS/arch covers CPython 3.8+ instead of needing one per
    version — turning a ~25-build matrix into 5.
  - Wheels are built for **linux x86_64 / aarch64, macOS x86_64 / arm64, and
    windows x64** via `PyO3/maturin-action`, which cross-compiles properly. A
    plain `maturin build` only ever targets the runner's own platform, which
    was the root cause. Both macOS wheels build on `macos-latest` (Apple
    Silicon), since GitHub has retired the `macos-13` Intel runner.
  - An **sdist** is built and uploaded as its own job, giving pip a source
    fallback on any platform without a prebuilt wheel.
- **Python metadata:** `pyproject.toml` advertised PyPy support that an abi3
  CPython extension cannot provide. Replaced with explicit CPython 3.8–3.13
  classifiers, so the metadata matches what is actually shipped.
- **CI:** bumped `actions/checkout` and `actions/setup-node` to v7 to move off
  the deprecated Node 20 runtime; the new wheel jobs use `actions/upload-artifact`
  v7 and `actions/download-artifact` v8. The `actions/setup-python` call site is
  gone — `maturin-action` provides the interpreter.
- **Parsers (HDFC BA):** Holder names starting with `MRS` were mangled into `S <name>` because the honorific was stripped by a plain substring replace. Honorifics are now removed as whole leading tokens.
- **Parsers (SBI BA):** The honorific (`Mrs.`, `Mr.` ...) is now stripped from the holder name instead of being kept as part of it, and the column padding statements use is collapsed.

## [0.3.0] - 2026-08-27

### Added
- **Parsers:** Added a new parser for Axis Bank credit card statements (`credit_cards/axis.rs`).
- **Web App:** Integrated the Axis Bank credit card parser in the UI.
- **Python / WASM:** Exposed the `parse_axis_cc` function in Python and WASM bindings.

## [0.2.4] - 2026-08-15

### Fixed
- **Parsers (CAMS):** Fixed summary-level validation false failures on statements where CAMS' vertical document-generation watermark (rotated 90°, stamped along the page margin) coincidentally landed within the y-tolerance of an AMC heading line, fusing a stray character onto it and causing that AMC's holdings to be silently misattributed to the previous AMC. Non-upright glyphs are now dropped during character extraction so they can never fuse onto content lines, regardless of which line they happen to land near.
- **Parsers (CAMS):** Fixed nondeterministic ordering of `summary_level` validation checks (backed by a `HashMap`, whose iteration order is randomized per process) by sorting by AMC name before emitting checks, so serialized output is stable across runs — the same class of issue already fixed for the IBKR parser in 0.2.0.
- **Error Handling:** CAMS PDF decryption failures now surface as the typed `XfinaError::IncorrectPassword` / `XfinaError::PasswordRequired` variants instead of a generic string-wrapped `ParseError`.

### Changed
- **Testing:** The CAMS integration test's `passwords.json` lookup now accepts either a single password or an ordered list of candidate passwords per key (including `default`), trying each in turn until one succeeds. Supports CAMS rotating its PDF password over time without needing per-file or date-based entries.

## [0.2.3] - 2026-08-09

### Added
- **Web App:** Added a privacy-first Analytics Consent modal with 3 tracking levels (Off, Page View, Parser Usage).

### Fixed
- **CI/CD:** Fixed publish workflow skipping jobs on tag push due to missing remote tracking branch in GitHub Actions checkout.

## [0.2.2] - 2026-08-08

### Added
- **xtask:** Split release command into `prepare-release` and `tag-release` stages to support PR-based branch protection workflows.

### Changed
- Add `homepage` to package metadata pointing to `xfina.dev`
- Fix page title to just `Xfina`

## [0.2.1] - 2026-08-08

### Added
- **Parsers:** Added derived `computed_closing_balance` validation logic for Bank of Baroda statements.
- **Parsers:** Added `overall_invested_match` and `overall_value_match` summary checks for Mutual Funds (CAS) by extracting portfolio summaries.

### Changed
- **Web App:** Standardized the UI header components across all statement types and added visual validation badges.

### Fixed
- **Web App:** Corrected UI alignment issues for IBKR transaction badges and properly greyed out non-applicable row-level validations for Credit Cards.

## [0.2.0] - 2026-08-06

### Added
- **Validation Engine:** Added a comprehensive two-level validation engine (`src/models/validation.rs`) to detect parsing discrepancies.
  - Row-level validation checks `opening balance + transaction amount = current balance`.
  - Summary-level validation checks `computed_closing = declared_closing` and verifies total credits/debits against declared summaries in PDFs/XLS files.
- **CI/CD:** Added GitHub Actions test pipeline (`.github/workflows/test.yml`) to automatically run `cargo test` and compile the WASM build on pushes and pull requests to `main`.
- **Documentation:** Created a comprehensive `CONTRIBUTING.md` guide for adding new parsers and managing snapshots.
- **Documentation:** Added an architectural diagram to the main `README.md` and added rich metadata (keywords, categories, readme) to `Cargo.toml`.

### Changed
- **Breaking API Change:** All parsers across all crates (`bank_accounts`, `credit_cards`, `mutual_funds`, `intl_stocks`) now return a wrapped `ParseResult<T>` struct containing the parsed `data: T` alongside a `validation: ValidationReport` object, instead of returning the raw account `T` directly.
- **WASM / Python / CLI Output:** The output JSON schema is now wrapped in `{ "data": { ... }, "validation": { ... } }`.
- **Error Handling:** Completely re-architected error handling across all parsers using the `thiserror` crate. Parsers now return a strongly-typed `XfinaError` enum instead of stringly-typed errors, enabling programmatic error matching.
- **Bindings:** Updated FFI boundaries in `xfina-wasm` (JS) and `xfina-py` (Python) to properly propagate `XfinaError` types.
- **Web App:** Updated the website header to explicitly link to `sakthipriyan.com/building-wealth` instead of linking generically to GitHub.

### Fixed
- **Code Cleanup:** Resolved hundreds of compiler and clippy warnings across the workspace, including unused variables, non-idiomatic default struct reassignments, and dead code.
- **Testing:** Resolved integration test failures in CI by ensuring snapshot write/assertion tests are skipped via a `GITHUB_ACTIONS=true` environment check.
- **Testing:** Replaced `HashMap` with `BTreeMap` and `HashSet` with `BTreeSet` in the IBKR parser to ensure deterministic serialization order for consistent snapshot tests.
- **Code Cleanup:** Removed legacy, unused `f64`-based financial models (`Portfolio`, `Asset`, etc.) from `src/models/mod.rs`.
- **Documentation:** Fixed an inaccuracy in `wasm/README.md` to correctly state that the default parser output format is `"xfina"`, not `"rebit"`.


## [0.1.4] - 2026-08-05

### Added
- **Documentation:** Added package registry badges (Crates.io, PyPI, npm) to the `README.md`.
- **Documentation:** Updated deployment instructions to reflect the new `xtask deploy-site` flow.

### Fixed
- **CI/CD:** Updated the release script in `xtask` to ensure `Cargo.lock` is correctly synced and to safely allow dirty publishing.

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

