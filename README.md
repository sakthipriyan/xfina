# Xfina

**Xfina** is a collection of libraries (Rust, Python, JS) and a web interface for extracting structured financial data from **Indian** bank statements, credit card statements, mutual fund reports, and international brokerage reports.

All parsers are compiled to **WebAssembly (WASM)** and run entirely in the browser — your financial data never leaves your device. 

🌐 **Live App**: [xfina.sakthipriyan.com](https://xfina.sakthipriyan.com/)

---

## Motivation & Vision

Most open-source financial parsers are written in Python, which requires users to set up a local toolchain and use a command-line interface (CLI), making them difficult to use directly via a web interface.

By building Xfina in **Rust**, we achieve:

1. **Privacy-first WASM Deployment** — WebAssembly (WASM) enables privacy-first tools that can run efficiently in the user's browser without sending sensitive financial data to any server. This zero-setup, browser-based solution empowers anyone who is comfortable with a web browser, Excel, or Google Sheets to easily extract a standardized data format without any technical overhead.
2. **Universal Bindings** — The goal is to support Python and JS bindings natively so the core logic can be used in any environment. We will start publishing to all 3 package systems (Rust crates, npm, and PyPI) once the Mutual Funds and IBKR parsers are fully wrapped up.
3. **ReBIT & Sahamati AA Standards** — The internal data schema is heavily built on top of the Sahamati Account Aggregator (AA) and ReBIT standards. Xfina offers a ready-made ReBIT JSON interface out-of-the-box, ensuring interoperability with standard Indian financial ecosystems.

---

## Supported Parsers & Status

### 🏦 Bank Accounts

| Crate | Institution | Format | Status | Notes |
|---|---|---|---|---|
| `xfina-ba-hdfc` | HDFC Bank | Excel (`.xls`/`.xlsx`) | **Production Ready** | Full support |
| `xfina-ba-icici` | ICICI Bank | Excel (`.xls`/`.xlsx`) | **Production Ready** | Full support |
| `xfina-ba-sbi` | State Bank of India | PDF (password protected) | **Production Ready** | Full support |
| `xfina-ba-bob` | Bank of Baroda | Excel (`.xls`/`.xlsx`) | **Production Ready** | Full support |

*Note: Parsers have not been tested with Joint Accounts.*

### 💳 Credit Cards

| Crate | Institution | Format | Status | Notes |
|---|---|---|---|---|
| `xfina-cc-hdfc` | HDFC Bank | CSV | **Production Ready** | Full support incl. add-on cardholders, reward points |
| `xfina-cc-icici` | ICICI Bank | Excel (`.xls`/`.xlsx`) | **Production Ready** | Tested card without any add-on cards |

### 📈 Mutual Funds

| Crate | Provider | Format | Status | Notes |
|---|---|---|---|---|
| `xfina-mf-cams` | CAMS | PDF (password protected) | **Needs more work** | Combined Account Statement (CAS) |

### 🌍 International Brokers

| Crate | Broker | Format | Status | Notes |
|---|---|---|---|---|
| `xfina-intl-stocks-ibkr` | Interactive Brokers (IBKR) | CSV | **WIP** | Activity statements |

---

## Architecture

The project is a **Cargo workspace** with these crates:

```
xfina/
├── models/               # xfina-models: shared data models (ReBIT / AA standard compatible)
├── bank-accounts/        # Bank Account parsers (HDFC, ICICI, SBI, BoB)
├── credit-cards/         # Credit Card parsers (HDFC, ICICI)
├── mutual-funds/         # Mutual Fund parsers (CAMS)
├── intl-stocks/          # International Broker parsers (IBKR)
├── wasm/                 # xfina-wasm: WASM bindings (wasm-bindgen)
└── web/                  # Vue 3 + Vite frontend (deployed via GitHub Pages)
```

### Data Models (`xfina-models`)

- **`CreditCardAccount`** — card details, statement period, account summary, transactions, reward points
- **`DepositAccount`** — account info, opening/closing balances, transactions
- **`Portfolio`** — investor info, assets, NAV, transactions (mutual funds & stocks)

All data structures inherently map to the Sahamati AA specifications, with project-specific extensions nested in the `xfina` object.

---

## Web App

The [`web/`](./web) directory contains a **Vue 3 + Vite** frontend that uses the WASM module to parse files directly in the browser.

### Features

- 🔒 **100% client-side** — no server, no uploads
- ⚡ **Rust/WASM performance** — parsing in milliseconds
- 📊 **Rich UI** — statement header, account summary, transaction table
- 🌙 **Dark mode** support
- 🏷️ **ReBIT compliance** — Direct JSON serialization into ReBIT structures

### Running Locally

```bash
# 1. Build WASM
cd wasm
wasm-pack build --target web && cp -r pkg/* ../web/src/wasm/

# 2. Start dev server
cd ../web
npm install
npm run dev
```

### Deployment

Pushed to `main` → GitHub Actions automatically builds WASM + Vue and deploys to GitHub Pages at [xfina.sakthipriyan.com](https://xfina.sakthipriyan.com/).

---

## Roadmap

### Initial Launch Targets

| Institution / Provider | Bank Account | Credit Card | Mutual Funds | Intl Brokers |
|---|:---:|:---:|:---:|:---:|
| Axis Bank | ✅ | | | |
| Bank of Baroda | ✅ | | | |
| CAMS | | | ✅ | |
| HDFC Bank | ✅ | ✅ | | |
| IBKR | | | | ✅ |
| ICICI Bank | ✅ | ✅ | | |
| State Bank of India | ✅ | | | |

### Over next few weeks
- [ ] CSV / JSON export in the web app
- [ ] Complete robust parsing for IBKR using 
- [ ] CAMS CAS (Mutual Funds)
- [ ] Add Bank Account Support for Axis Bank
- [ ] Publish `xfina` to Rust crates.io
- [ ] Publish JavaScript/TypeScript WASM bindings to npm
- [ ] Publish Python bindings (`xfina-py`) via `pyo3` to PyPI

## License

[Apache 2.0](./LICENSE)