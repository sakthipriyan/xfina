use std::collections::HashMap;
use financial_extract_models::{Portfolio, Asset, Transaction, InvestorInfo};
use csv::ReaderBuilder;
use chrono::{NaiveDate, NaiveDateTime, TimeZone, LocalResult, Utc};
use chrono_tz::America::New_York;

fn format_date_iso(date_str: &str) -> String {
    let clean_str = date_str.replace(" EDT", "").replace(" EST", "");
    
    if let Ok(dt) = NaiveDateTime::parse_from_str(&clean_str, "%Y-%m-%d, %H:%M:%S") {
        let ny_dt = match New_York.from_local_datetime(&dt) {
            LocalResult::None => return date_str.to_string(),
            LocalResult::Single(t) => t,
            LocalResult::Ambiguous(t, _) => t,
        };
        return ny_dt.with_timezone(&Utc).to_rfc3339();
    }
    
    if let Ok(d) = NaiveDate::parse_from_str(&clean_str, "%B %d, %Y") {
        return d.format("%Y-%m-%d").to_string();
    }

    if let Ok(d) = NaiveDate::parse_from_str(&clean_str, "%Y-%m-%d") {
        return d.format("%Y-%m-%d").to_string();
    }
    
    date_str.to_string()
}

pub fn parse_ibkr_csv(csv_content: &str) -> Result<Portfolio, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_content.as_bytes());

    let mut account_no = String::from("IBKR");
    let mut investor_name = None;
    let mut statement_start_date = None;
    let mut statement_end_date = None;
    let mut generated_date = None;

    // symbol -> (primary_symbol, description, isin)
    let mut instruments: HashMap<String, (String, String, String)> = HashMap::new();
    // symbol -> Vec<Transaction>
    let mut trades: HashMap<String, Vec<Transaction>> = HashMap::new();
    // symbol -> (quantity, value, cost_basis, close_price)
    let mut positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    // symbol -> prior_quantity
    let mut prior_quantities: HashMap<String, f64> = HashMap::new();

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
                    account_no = acc.to_string();
                }
            }
            (Some("Account Information"), Some("Data"), Some("Name")) => {
                if let Some(name) = record.get(3) {
                    investor_name = Some(name.to_string());
                }
            }
            (Some("Statement"), Some("Data"), Some("Period")) => {
                if let Some(period) = record.get(3) {
                    let parts: Vec<&str> = period.split('-').collect();
                    if parts.len() == 2 {
                        statement_start_date = Some(format_date_iso(parts[0].trim()));
                        statement_end_date = Some(format_date_iso(parts[1].trim()));
                    }
                }
            }
            (Some("Statement"), Some("Data"), Some("WhenGenerated")) => {
                if let Some(generated) = record.get(3) {
                    generated_date = Some(format_date_iso(generated.trim()));
                }
            }
            (Some("Financial Instrument Information"), Some("Data"), _) => {
                let symbols_str = record.get(3).unwrap_or("");
                let description = record.get(4).unwrap_or("").to_string();
                let isin = record.get(6).unwrap_or("").to_string();

                let mut primary_sym = String::new();
                for (i, sym) in symbols_str.split(',').enumerate() {
                    let sym = sym.trim();
                    if i == 0 {
                        primary_sym = sym.to_string();
                    }
                    if !sym.is_empty() {
                        instruments.insert(sym.to_string(), (primary_sym.clone(), description.clone(), isin.clone()));
                    }
                }
            }
            (Some("Mark-to-Market Performance Summary"), Some("Data"), Some("Stocks")) => {
                let symbol = record.get(3).unwrap_or("").to_string();
                let prior_qty: f64 = record.get(4).unwrap_or("0").parse().unwrap_or(0.0);
                if !symbol.is_empty() {
                    prior_quantities.insert(symbol, prior_qty);
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
                    let t_price: f64 = record.get(8).unwrap_or("0").parse().unwrap_or(0.0);
                    let proceeds: f64 = record.get(10).unwrap_or("0").parse().unwrap_or(0.0);
                    let comm_fee: f64 = record.get(11).unwrap_or("0").parse().unwrap_or(0.0);

                    let amount = proceeds + comm_fee;

                    if !symbol.is_empty() && !date.is_empty() && amount != 0.0 {
                        let tx_type = if quantity > 0.0 { "BUY".to_string() } else { "SELL".to_string() };
                        let tx = Transaction {
                            date: format_date_iso(&date),
                            tx_type,
                            amount: amount.abs(),
                            units: quantity.abs(),
                            nav: if t_price != 0.0 { Some(t_price) } else { None },
                            balance: None, // Will be calculated after sorting
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

    let get_primary = |sym: &str| -> String {
        if let Some((primary, _, _)) = instruments.get(sym) {
            primary.clone()
        } else {
            sym.to_string()
        }
    };

    let mut merged_positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    for (sym, pos) in positions {
        let primary = get_primary(&sym);
        let entry = merged_positions.entry(primary).or_insert((0.0, 0.0, 0.0, 0.0));
        entry.0 += pos.0;
        entry.1 += pos.1;
        entry.2 += pos.2;
        entry.3 = pos.3; // Just take the last close price
    }

    let mut merged_trades: HashMap<String, Vec<Transaction>> = HashMap::new();
    for (sym, mut txs) in trades {
        let primary = get_primary(&sym);
        merged_trades.entry(primary).or_default().append(&mut txs);
    }

    let mut merged_prior: HashMap<String, f64> = HashMap::new();
    for (sym, qty) in prior_quantities {
        let primary = get_primary(&sym);
        *merged_prior.entry(primary).or_insert(0.0) += qty;
    }

    let mut all_symbols = std::collections::HashSet::new();
    for sym in merged_positions.keys() { all_symbols.insert(sym.clone()); }
    for sym in merged_trades.keys() { all_symbols.insert(sym.clone()); }
    for sym in merged_prior.keys() { all_symbols.insert(sym.clone()); }

    // Assemble Portfolio
    for symbol in all_symbols {
        let (desc, isin) = if let Some((_, d, i)) = instruments.get(&symbol) {
            (d.clone(), i.clone())
        } else {
            (symbol.clone(), "".to_string())
        };
        
        let pos = merged_positions.get(&symbol).cloned().unwrap_or((0.0, 0.0, 0.0, 0.0));
        let mut txs = merged_trades.remove(&symbol).unwrap_or_default();
        
        // Sort transactions by date (simple string sort for now, might need datetime parsing)
        txs.sort_by(|a, b| a.date.cmp(&b.date));

        // Calculate running balance and invested value
        let mut current_balance = merged_prior.get(&symbol).cloned().unwrap_or(0.0);
        let mut invested = 0.0;
        
        for tx in txs.iter_mut() {
            if tx.tx_type == "BUY" {
                current_balance += tx.units;
                invested += tx.amount;
            } else if tx.tx_type == "SELL" {
                current_balance -= tx.units;
            }
            tx.balance = Some(current_balance);
        }

        let total_units = pos.0;
        let current_value = pos.1;
        let close_price = pos.3;

        assets.push(Asset {
            name: desc,
            isin: if isin.is_empty() { None } else { Some(isin) },
            symbol: Some(symbol),
            category: None,
            total_units,
            invested_value: invested,
            current_nav: if close_price != 0.0 { Some(close_price) } else { None },
            current_nav_date: statement_end_date.clone(),
            current_value: if current_value != 0.0 { Some(current_value) } else { None },
            transactions: txs,
        });
    }

    let investor_info = InvestorInfo {
        account_number: Some(account_no),
        name: investor_name,
        email: None,
        pan: None,
        contact: None,
        address: None,
    };

    Ok(Portfolio { 
        investor_info,
        statement_start_date,
        statement_end_date,
        generated_date,
        assets 
    })
}
