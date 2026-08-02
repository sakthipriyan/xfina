# Xfina (WASM Bindings)

[![npm version](https://badge.fury.io/js/xfina.svg)](https://badge.fury.io/js/xfina)

**Xfina** is a blazingly fast WebAssembly (WASM) library for parsing Indian financial statements (Bank Accounts, Credit Cards, Mutual Funds) and international brokers (IBKR).

It runs **100% in the browser** or Node.js, extracting structured data from raw PDFs, Excel files, and CSVs into ReBIT-compliant JSON objects without ever sending sensitive financial data to a server.

## Installation

```bash
npm install xfina-wasm
```

## Quick Start (Browser / Vite)

```javascript
import init, { parse_hdfc_ba, parse_cams } from 'xfina-wasm';

async function parseStatement(file) {
  // Initialize the WASM module
  await init();

  // Read the file as an ArrayBuffer, then convert to Uint8Array
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);

  try {
    // Parse the statement!
    const accountData = parse_hdfc_ba(bytes, null);
    console.log(accountData);
  } catch (error) {
    console.error("Failed to parse statement:", error);
  }
}
```

## Available Parsers

All parsers return a structured JavaScript object that mirrors the ReBIT JSON schema.

| Category | Institution | Format | JS Function | Input Type |
|---|---|---|---|---|
| Bank Account | HDFC | `.xls` | `parse_hdfc_ba(bytes, password)` | `Uint8Array` |
| Bank Account | ICICI | `.xls` | `parse_icici_ba(bytes, filename)` | `Uint8Array` |
| Bank Account | SBI | PDF | `parse_sbi_ba(bytes, password, filename)` | `Uint8Array` |
| Bank Account | BOB | `.xls` | `parse_bob_ba(bytes)` | `Uint8Array` |
| Bank Account | Axis | `.xls` | `parse_axis_ba(bytes, filename)` | `Uint8Array` |
| Credit Card | HDFC | CSV | `parse_hdfc_cc(content, filename)` | `String` |
| Credit Card | ICICI | `.xls` | `parse_icici_cc(bytes, filename)` | `Uint8Array` |
| Mutual Funds | CAMS | PDF | `parse_cams(bytes, password, filename)` | `Uint8Array` |
| Intl Stocks | IBKR | CSV | `parse_ibkr(content)` | `String` |

*Note: For `parse_hdfc_cc` and `parse_ibkr`, you must read the file as text and pass a JavaScript `String` instead of a `Uint8Array`.*
