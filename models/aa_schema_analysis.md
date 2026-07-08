# RBI Account Aggregator (FI) Schema Applicability Analysis

## Overview
This analysis compares the internal models of Xfina (`xfina-models`) with the standard Financial Information (FI) data schemas defined by ReBIT (Reserve Bank Information Technology) for the Indian Account Aggregator (AA) ecosystem.

ReBIT defines standardized XML/JSON schemas for various FI types. All schemas generally follow a tripartite structure:
1. **Profile**: Account holder metadata (name, PAN, DOB, etc.)
2. **Summary**: High-level snapshot (balances, account status, etc.)
3. **Transactions**: Array of chronological transaction logs.

---

## 1. Bank Accounts (Deposit Schema)

**ReBIT `DEPOSIT` Schema (v2.0.0)**
- **Profile**: `Holders` (name, dob, pan, email, etc.)
- **Summary**: `currentBalance`, `currency`, `balanceDateTime`, `openingDate`, `status`, `accountType` (SAVINGS, CURRENT).
- **Transactions**: `txnId`, `amount`, `date`, `valueDate`, `type` (CREDIT/DEBIT), `narration`, `currentBalance`.

**Xfina `BankAccountStatement`**
- **Profile**: Captured in `customer_info` (name, address, gstn).
- **Summary**: Captures `opening_balance`, `closing_balance`, `total_credits`, `total_debits`.
- **Transactions**: `BankTransaction` handles `date`, `value_date`, `description` (narration), `tx_type`, `amount`, `balance`.

> [!TIP]
> **Applicability & Gaps**: High compatibility. To map directly to AA JSON output, we would need to standardise our `CustomerInfo` to include PAN, Email, DOB (where available on the statement), and assign an explicit `accountType` and `currency` (currently implicit in the UI). We are also missing a unique `txnId` because physical statements don't provide them; we'd need to generate synthetic hashes (e.g., SHA256 of date+amount+narration).

---

## 2. Credit Cards (Credit Card Schema)

**ReBIT `CREDIT_CARD` Schema**
- **Profile**: Primary cardholder details.
- **Summary**: `creditLimit`, `availableCredit`, `currentDue`, `dueDate`, `previousBalance`, etc.
- **Transactions**: Basic transaction log (date, amount, narration, MCC).

**Xfina `CreditCardStatement`**
- **Profile**: Handled by `customer_info` and `card_no`.
- **Summary**: Includes `credit_limit`, `available_limit`, `total_amount_due`, `payment_due_date`, and deep breakdowns (`AccountSummary`, `PastDues`, `RewardPointsSummary`).
- **Transactions**: `CreditCardTransaction` handles `date`, `description`, `amount`, `tx_type`, `reward_points`.

> [!NOTE]
> **Applicability & Gaps**: Xfina's schema is actually **richer** than the standard ReBIT Credit Card schema in some respects (especially in multi-card breakdown and reward points extraction). Exporting to the ReBIT schema would be a "lossy" conversion for rewards, but the core financial fields map 1:1.

---

## 3. Mutual Funds (Mutual Funds Schema)

**ReBIT `MUTUAL_FUNDS` Schema**
- **Profile**: Investor details (PAN, KYC status, nominee).
- **Summary**: Investment summary (total investment, current value, total units).
- **Transactions**: `AMC` level groupings -> `Scheme` level groupings -> Transactions (amount, nav, units, type).

**Xfina `Portfolio` / `Asset`**
- **Profile**: `InvestorInfo` handles name, pan, email, address, contact.
- **Summary**: Rolled up at the `Asset` level (`total_cost_basis`, `current_value`, `total_units`).
- **Transactions**: `Transaction` handles amount, units, nav, fee, balance.

> [!WARNING]
> **Applicability & Gaps**: Xfina groups by `Asset` directly rather than standardizing an `AMC` -> `Scheme` hierarchy required by ReBIT. The AA schema relies heavily on ISINs; Xfina extracts ISINs when present (e.g., CAMS PDF), but if not present, the mapping will lack standard identifiers required by downstream FIUs.

## 4. Equities / Stocks (Equities Schema)

**ReBIT `EQUITIES` Schema**
- **Profile**: Investor details (PAN, Demat Account Number, DP ID).
- **Summary**: Holdings snapshot (investment value, current market value).
- **Transactions**: Exchange-level or depository-level transaction records (buy/sell, corporate actions) typically identified by ISIN.

**Xfina `Portfolio` / `Asset` (for IBKR)**
- **Profile**: `InvestorInfo` captures account number, name, and address.
- **Summary**: Aggregated similarly to mutual funds (cost basis, current value, total units).
- **Transactions**: `Transaction` captures date, type, amount, units, price, and fee.

> [!TIP]
> **Applicability & Gaps**: Great news! The `ISIN` is actually provided in the IBKR statements under the "Financial Instrument Information" section as the `Security ID` (e.g., `IE000QDFFK00`). 
> 
> Because we have the ISIN, **all other fields match the schema almost perfectly**:
> - **Holdings**: Mapped from the "Open Positions" section (Cost Basis, Value).
> - **Transactions**: Mapped directly from the "Trades" section (Buy/Sell, Amount, Date, Price).
> - **Profile**: We can map the Account Name.
> 
> To fully conform to the strict ReBIT AA schema for Equities, we can map the IBKR `Account` (e.g., `U20010225`) directly to the `Client ID` field. We can simply leave `DP ID` blank (or use a placeholder like "IBKR"), since that is an Indian Depository construct. 

---

## Conclusion & Architectural Decisions

Xfina's internal data models are structurally very similar to the ReBIT AA schemas, making it highly feasible to adopt them as our core structure. 

### 1. Schema Unification (The "AA Superset")
Rather than maintaining our own internal naming conventions (e.g., `description`) and mapping them to ReBIT (e.g., `narration`) during export, **we will unify `xfina-models` to natively use the ReBIT AA schema field names.**
- **Direct Alignment**: If a concept exists in the AA schema, we use their exact field name (e.g., `txnId`, `valueDate`, `narration`).
- **Extended Fields**: For data not covered by ReBIT (e.g., Credit Card reward points, multi-card sub-allocations), we will append our own custom fields to the models.
- **Graceful Degradation**: Missing regulatory details (like `DP ID` for foreign brokers) will simply be left blank/null.

This makes Xfina natively AA-compliant out of the box, with zero translation overhead.

### 2. CSV Export Strategy
While JSON handles nested data (Profile -> Summary -> Transactions) perfectly, CSV requires flattening. For CSV exports, we will adopt a **Flat Transaction-Centric** approach:
- **Focus on Transactions**: The primary rows of the CSV will be the `Transactions` array, as this is what users typically want to analyze in Excel or import into accounting software (like Tally or portfolio trackers).
- **Denormalized Metadata**: We will flatten the `Profile` and `Summary` data by appending key fields (e.g., `Account Number`, `Account Type`, `Currency`) as prefix columns on *every* transaction row. 
- **Extended Columns**: Any custom Xfina fields (like `Reward Points`) will just become additional columns at the end of the row. 

This approach ensures the CSV is immediately useful for pivot tables and standard financial software without complex relational parsing.
