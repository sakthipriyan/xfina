# Xfina Models (`xfina-models`)

Shared data structures for **Xfina**. 

These models define how financial statements (Bank Accounts, Credit Cards, Mutual Funds, International Stocks) are structured internally, and how they serialize into JSON.

## Standards Compliance

The structures natively serialize into **ReBIT (Reserve Bank Information Technology)** JSON schemas. This ensures interoperability with the Sahamati Account Aggregator (AA) framework used by Indian financial institutions.

Each model has project-specific extensions nested in an `xfina` object to preserve data that might not cleanly fit into the ReBIT schema but is still valuable for downstream analysis (like detailed reward point histories for credit cards or specific transaction categories).

## Core Models

- `DepositAccount`: For savings, current, and overdraft bank accounts.
- `CreditCardAccount`: For credit card statements.
- `MutualFundsAccount`: For CAMS CAS and other mutual fund statements.
- `EquityAccount`: For international brokers like IBKR.
