# FinX

**FinX** is a collection of Rust libraries for extracting structured financial data from bank statements, credit card statements, mutual fund reports, and international brokerage reports.

All parsers are compiled to **WebAssembly (WASM)** and run entirely in the browser — your financial data never leaves your device.

🌐 **Live App**: [finx.sakthipriyan.com](https://sakthipriyan.github.io/financial-extract/)

---

## Motivation & Vision

Most open-source financial parsers (e.g. `casparser`, `processCASpdf`) are written in Python, which requires a backend server to process files — raising significant privacy concerns for sensitive financial documents.

By building FinX in **Rust**, we achieve:

1. **Privacy-first WASM Deployment** — Parsers compile to WebAssembly and run *entirely* in the user's browser. Data never leaves the device.
2. **Backend & Data Science via Python** — Using `pyo3`, the same Rust engine can be exposed to Python for high-performance pipelines.
3. **Modularity & Extensibility** — A unified data model across all parsers makes it easy to add new institutions.

---

## Supported Parsers

### 💳 Credit Cards

| Crate | Institution | Format | Notes |
|---|---|---|---|
| `finx-cc-hdfc` | HDFC Bank | CSV | Full support incl. add-on cardholders, reward points |
| `finx-cc-icici` | ICICI Bank | Excel (`.xls`/`.xlsx`) | Single & international transactions |

### 🏦 Bank Accounts

| Crate | Institution | Format | Notes |
|---|---|---|---|
| `finx-ba-hdfc` | HDFC Bank | Excel (`.xls`/`.xlsx`) | Full support |
| `finx-ba-icici` | ICICI Bank | Excel (`.xls`/`.xlsx`) | Date extracted from filename |
| `finx-ba-sbi` | State Bank of India | PDF (password protected) | Full support |
| `finx-ba-bob` | Bank of Baroda | Excel (`.xls`/`.xlsx`) | Basic support |

### 📈 Mutual Funds

| Crate | Provider | Format | Notes |
|---|---|---|---|
| `finx-mf-cams` | CAMS | PDF (password protected) | Combined Account Statement |

### 🌍 International Stocks

| Crate | Broker | Format | Notes |
|---|---|---|---|
| `finx-intl-stocks-ibkr` | Interactive Brokers (IBKR) | CSV | Activity statements |

---

## Architecture

The project is a **Cargo workspace** with these crates:

```
financial-extract/
├── models/               # finx-models: shared data models (Portfolio, BankStatement, CreditCardStatement, etc.)
├── mutual-funds/
│   └── cams/             # finx-mf-cams: CAMS CAS PDF parser
├── intl-stocks/
│   └── ibkr/             # finx-intl-stocks-ibkr: IBKR CSV parser
├── credit-cards/
│   ├── hdfc/             # finx-cc-hdfc: HDFC credit card CSV parser
│   └── icici/            # finx-cc-icici: ICICI credit card Excel parser
├── bank-accounts/
│   ├── hdfc/             # finx-ba-hdfc: HDFC bank account Excel parser
│   ├── icici/            # finx-ba-icici: ICICI bank account Excel parser
│   ├── sbi/              # finx-ba-sbi: SBI bank account PDF parser
│   └── bob/              # finx-ba-bob: Bank of Baroda Excel parser
├── wasm/                 # finx-wasm: WASM bindings (wasm-bindgen)
└── web/                  # Vue 3 + Vite frontend (deployed via GitHub Pages)
```

### Data Models (`finx-models`)

- **`CreditCardStatement`** — card details, statement period, account summary, transactions, reward points
- **`BankStatement`** — account info, opening/closing balances, transactions
- **`Portfolio`** — investor info, assets, NAV, transactions (mutual funds & stocks)

All date fields use ISO 8601 format (`YYYY-MM-DD`). Derived dates (inferred from transactions when not explicitly stated in the file) are flagged via `*_derived: bool` fields.

---

## Web App

The [`web/`](./web) directory contains a **Vue 3 + Vite** frontend that uses the WASM module to parse files directly in the browser.

### Features

- 🔒 **100% client-side** — no server, no uploads
- ⚡ **Rust/WASM performance** — parsing in milliseconds
- 📊 **Rich UI** — statement header, account summary, transaction table
- 🏷️ **Derived date indicator** — `est.` badge on dates inferred from transactions
- 🌙 **Dark mode** support
- 📤 **Export** — CSV / JSON (coming soon)

### Running Locally

```bash
# 1. Build WASM
wasm-pack build wasm --target web --out-dir ../web/src/wasm

# 2. Start dev server
cd web
npm install
npm run dev
```

### Deployment

Pushed to `main` → GitHub Actions automatically builds WASM + Vue and deploys to GitHub Pages.

---

## Current Status

| Parser | Status | Notes |
|---|---|---|
| `finx-cc-hdfc` | ✅ Stable | Tested with add-on cards; international not verified |
| `finx-cc-icici` | ✅ Stable | Domestic tested |
| `finx-ba-hdfc` | ✅ Stable | |
| `finx-ba-icici` | ✅ Stable | |
| `finx-ba-sbi` | ✅ Stable | Password-protected PDF |
| `finx-ba-bob` | 🚧 Beta | Basic support |
| `finx-mf-cams` | 🚧 WIP | PDF extraction needs more work |
| `finx-intl-stocks-ibkr` | ✅ Mostly stable | |

---

## Roadmap

- [ ] Python bindings (`finx-py`) via `pyo3`
- [ ] CSV / JSON export in the web app
- [ ] Support for more banks and institutions
- [ ] Axis Bank credit card / bank account
- [ ] Kotak Bank statements

---

## License

[Apache 2.0](./LICENSE)