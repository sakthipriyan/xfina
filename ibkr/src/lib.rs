use std::collections::HashMap;
use financial_extract_models::{Portfolio, Asset, Transaction};
use csv::ReaderBuilder;

pub fn parse_ibkr_csv(csv_content: &str) -> Result<Portfolio, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_content.as_bytes());

    let mut _account_no = String::from("IBKR");
    let mut _investor_name = None;

    // symbol -> (description, isin)
    let mut instruments: HashMap<String, (String, String)> = HashMap::new();
    // symbol -> Vec<Transaction>
    let mut trades: HashMap<String, Vec<Transaction>> = HashMap::new();
    // symbol -> (quantity, value, cost_basis, close_price)
    let mut positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        if record.len() < 3 {
            continue;
        }

        match (record.get(0), record.get(1), record.get(2)) {
            (Some("Account Information"), Some("Data"), Some("Account")) => {
                if let Some(acc) = record.get(3) {
                    _account_no = acc.to_string();
                }
            }
            (Some("Account Information"), Some("Data"), Some("Name")) => {
                if let Some(name) = record.get(3) {
                    _investor_name = Some(name.to_string());
                }
            }
            (Some("Financial Instrument Information"), Some("Data"), _) => {
                let symbols_str = record.get(3).unwrap_or("");
                let description = record.get(4).unwrap_or("").to_string();
                let isin = record.get(6).unwrap_or("").to_string();

                for sym in symbols_str.split(',') {
                    let sym = sym.trim();
                    if !sym.is_empty() {
                        instruments.insert(sym.to_string(), (description.clone(), isin.clone()));
                    }
                }
            }
            (Some("Open Positions"), Some("Data"), Some("Summary")) => {
                if record.get(3) == Some("Stocks") {
                    let symbol = record.get(5).unwrap_or("").to_string();
                    let quantity: f64 = record.get(6).unwrap_or("0").parse().unwrap_or(0.0);
                    let cost_basis: f64 = record.get(9).unwrap_or("0").parse().unwrap_or(0.0);
                    let close_price: f64 = record.get(10).unwrap_or("0").parse().unwrap_or(0.0);
                    let value: f64 = record.get(11).unwrap_or("0").parse().unwrap_or(0.0);

                    if !symbol.is_empty() {
                        positions.insert(symbol, (quantity, value, cost_basis, close_price));
                    }
                }
            }
            (Some("Trades"), Some("Data"), Some("Order")) => {
                if record.get(3) == Some("Stocks") {
                    let symbol = record.get(5).unwrap_or("").to_string();
                    let date = record.get(6).unwrap_or("").to_string();
                    let quantity: f64 = record.get(7).unwrap_or("0").parse().unwrap_or(0.0);
                    let proceeds: f64 = record.get(10).unwrap_or("0").parse().unwrap_or(0.0);
                    let comm_fee: f64 = record.get(11).unwrap_or("0").parse().unwrap_or(0.0);

                    let amount = proceeds + comm_fee;

                    if !symbol.is_empty() && !date.is_empty() && amount != 0.0 {
                        let tx_type = if quantity > 0.0 { "BUY".to_string() } else { "SELL".to_string() };
                        let tx = Transaction {
                            date,
                            tx_type,
                            amount: amount.abs(),
                            units: quantity.abs(),
                            nav: None, // usually trade price, but not directly requested
                            balance: None,
                            fee: Some(comm_fee),
                        };

                        trades.entry(symbol).or_insert_with(Vec::new).push(tx);
                    }
                }
            }
            _ => {}
        }
    }

    let mut assets = Vec::new();

    // Assemble Portfolio
    for (symbol, _pos) in positions {
        let (desc, isin) = instruments.get(&symbol).cloned().unwrap_or_else(|| (symbol.clone(), "".to_string()));
        let mut txs = trades.remove(&symbol).unwrap_or_default();
        
        // Sort transactions by date (simple string sort for now, might need datetime parsing)
        txs.sort_by(|a, b| a.date.cmp(&b.date));

        assets.push(Asset {
            name: desc,
            isin: if isin.is_empty() { None } else { Some(isin) },
            symbol: Some(symbol),
            category: None,
            transactions: txs,
        });
    }

    Ok(Portfolio { assets })
}
