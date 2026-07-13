use std::collections::HashMap;
use xfina_models::{
    EquityAccount, EquityProfile, EquityHolders, EquityHolder, EquitySummary, EquityInvestment,
    EquityHoldings, EquityHolding, EquityTransactions, EquityTransaction, XfinaEquityAccount,
    EquityFiType, ShareHolderEquityType, EquityTransactionType, TransactionsSymbol, 
    EquityCategory,
};
use csv::ReaderBuilder;
use chrono::{NaiveDate, NaiveDateTime, TimeZone, LocalResult, Utc};
use chrono_tz::America::New_York;
use rust_decimal::Decimal;

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

fn parse_date(date_str: &str) -> Option<NaiveDate> {
    let iso = format_date_iso(date_str);
    if iso.len() >= 10 {
        NaiveDate::parse_from_str(&iso[..10], "%Y-%m-%d").ok()
    } else {
        None
    }
}

fn parse_datetime(date_str: &str) -> Option<chrono::DateTime<Utc>> {
    let iso = format_date_iso(date_str);
    chrono::DateTime::parse_from_rfc3339(&iso).map(|d| d.with_timezone(&Utc)).ok()
}

pub fn parse_ibkr_csv(csv_content: &str) -> Result<EquityAccount, String> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_content.as_bytes());

    let mut account_no = String::from("IBKR");
    let mut investor_name = String::new();
    let mut statement_start_date = None;
    let mut statement_end_date = None;
    let mut generated_date = None;

    // symbol -> (primary_symbol, description, isin)
    let mut instruments: HashMap<String, (String, String, String)> = HashMap::new();
    
    // Interim trades representation since EquityTransaction expects Decimal
    struct InterimTrade {
        date: String,
        tx_type: EquityTransactionType,
        amount: Decimal,
        units: Decimal,
        t_price: Decimal,
        comm_fee: Decimal,
    }
    // symbol -> Vec<InterimTrade>
    let mut trades: HashMap<String, Vec<InterimTrade>> = HashMap::new();
    
    // symbol -> (quantity, value, cost_basis, close_price)
    let mut positions: HashMap<String, (Decimal, Decimal, Decimal, Decimal)> = HashMap::new();

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
                    investor_name = name.to_string();
                }
            }
            (Some("Statement"), Some("Data"), Some("Period")) => {
                if let Some(period) = record.get(3) {
                    let parts: Vec<&str> = period.split('-').collect();
                    if parts.len() == 2 {
                        statement_start_date = parse_date(parts[0].trim());
                        statement_end_date = parse_date(parts[1].trim());
                    }
                }
            }
            (Some("Statement"), Some("Data"), Some("WhenGenerated")) => {
                if let Some(generated) = record.get(3) {
                    generated_date = parse_datetime(generated.trim());
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
            (Some("Open Positions"), Some("Data"), Some("Summary")) => {
                if record.get(3) == Some("Stocks") {
                    let symbol = record.get(5).unwrap_or("").to_string();
                    let quantity: Decimal = record.get(6).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);
                    let cost_basis: Decimal = record.get(9).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);
                    let close_price: Decimal = record.get(10).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);
                    let value: Decimal = record.get(11).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);

                    if !symbol.is_empty() {
                        positions.insert(symbol, (quantity, value, cost_basis, close_price));
                    }
                }
            }
            (Some("Trades"), Some("Data"), Some("Order")) => {
                if record.get(3) == Some("Stocks") {
                    let symbol = record.get(5).unwrap_or("").to_string();
                    let date = record.get(6).unwrap_or("").to_string();
                    let quantity: Decimal = record.get(7).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);
                    let t_price: Decimal = record.get(8).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);
                    let proceeds: Decimal = record.get(10).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);
                    let comm_fee: Decimal = record.get(11).unwrap_or("0").parse().unwrap_or(Decimal::ZERO);

                    let amount = proceeds + comm_fee;

                    if !symbol.is_empty() && !date.is_empty() && amount != Decimal::ZERO {
                        let tx_type = if quantity > Decimal::ZERO { EquityTransactionType::Buy } else { EquityTransactionType::Sell };
                        let tx = InterimTrade {
                            date: format_date_iso(&date),
                            tx_type,
                            amount: amount.abs(),
                            units: quantity.abs(),
                            t_price,
                            comm_fee,
                        };

                        trades.entry(symbol).or_insert_with(Vec::new).push(tx);
                    }
                }
            }
            _ => {}
        }
    }

    let mut final_holdings = Vec::new();
    let mut final_transactions = Vec::new();
    let mut total_investment_value = Decimal::ZERO;
    let mut total_current_value = Decimal::ZERO;

    let get_primary = |sym: &str| -> String {
        if let Some((primary, _, _)) = instruments.get(sym) {
            primary.clone()
        } else {
            sym.to_string()
        }
    };

    let mut merged_positions: HashMap<String, (Decimal, Decimal, Decimal, Decimal)> = HashMap::new();
    for (sym, pos) in positions {
        let primary = get_primary(&sym);
        let entry = merged_positions.entry(primary).or_insert((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO));
        entry.0 += pos.0;
        entry.1 += pos.1;
        entry.2 += pos.2;
        entry.3 = pos.3; // Just take the last close price
    }

    let mut merged_trades: HashMap<String, Vec<InterimTrade>> = HashMap::new();
    for (sym, mut txs) in trades {
        let primary = get_primary(&sym);
        merged_trades.entry(primary).or_default().append(&mut txs);
    }

    let mut all_symbols = std::collections::HashSet::new();
    for sym in merged_positions.keys() { all_symbols.insert(sym.clone()); }
    for sym in merged_trades.keys() { all_symbols.insert(sym.clone()); }

    let mut txn_counter = 1;

    for symbol in all_symbols {
        let (desc, isin) = if let Some((_, d, i)) = instruments.get(&symbol) {
            (d.clone(), i.clone())
        } else {
            (symbol.clone(), "".to_string())
        };
        
        let pos = merged_positions.get(&symbol).cloned().unwrap_or((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO));
        let mut txs = merged_trades.remove(&symbol).unwrap_or_default();
        
        txs.sort_by(|a, b| a.date.cmp(&b.date));

        let mut period_invested_value = Decimal::ZERO;
        let mut period_realized_value = Decimal::ZERO;
        let mut tx_qty_diff = Decimal::ZERO;
        let mut period_buy_units = Decimal::ZERO;
        let mut period_sell_units = Decimal::ZERO;
        let mut period_buy_count = 0;
        let mut period_sell_count = 0;

        for tx in &txs {
            let tx_date = chrono::DateTime::parse_from_rfc3339(&tx.date).map(|d| d.with_timezone(&Utc)).ok();
            
            if tx.tx_type == EquityTransactionType::Buy {
                tx_qty_diff += tx.units;
                period_buy_units += tx.units;
                period_buy_count += 1;
                period_invested_value += tx.amount;
            } else {
                tx_qty_diff -= tx.units;
                period_sell_units += tx.units;
                period_sell_count += 1;
                period_realized_value += tx.amount;
            }

            final_transactions.push(EquityTransaction {
                txn_id: format!("IBKR-{}", txn_counter),
                order_id: None,
                trade_id: None,
                company_name: Some(desc.clone()),
                symbol: Some(symbol.clone()),
                transaction_date_time: tx_date,
                exchange: Some(TransactionsSymbol::Others),
                isin: if isin.is_empty() { None } else { Some(isin.clone()) },
                equity_category: Some(EquityCategory::Equity),
                instrument_type: None,
                option_type: None,
                strike_price: None,
                narration: Some("Trade".to_string()),
                rate: Some(tx.t_price.abs()),
                total_charge: Some(tx.comm_fee.abs()),
                trade_value: Some(tx.amount),
                r#type: tx.tx_type,
                share_holder_equity_type: Some(ShareHolderEquityType::CommonStock),
                units: tx.units,
                other_charges: None,
            });
            txn_counter += 1;
        }

        let total_units = pos.0;
        let current_value = pos.1;
        let opening_balance = total_units - tx_qty_diff;
        let total_cost_basis = pos.2;
        let close_price = pos.3;

        total_investment_value += total_cost_basis;
        total_current_value += current_value;

        if total_units > Decimal::ZERO {
            final_holdings.push(EquityHolding {
                issuer_name: desc.clone(),
                isin: isin.clone(),
                units: total_units,
                investment_date_time: None,
                rate: if total_units != Decimal::ZERO { Some(total_cost_basis / total_units) } else { None },
                last_traded_price: Some(close_price),
                description: Some(symbol.clone()),
                xfina: Some(xfina_models::equity::XfinaEquityHolding {
                    opening_balance: Some(opening_balance),
                    closing_balance: Some(total_units),
                    period_invested_value: Some(period_invested_value),
                    period_realized_value: Some(period_realized_value),
                    period_buy_units: Some(period_buy_units),
                    period_sell_units: Some(period_sell_units),
                    period_buy_count: Some(period_buy_count),
                    period_sell_count: Some(period_sell_count),
                }),
            });
        }
    }

    let holder = EquityHolder {
        name: if investor_name.is_empty() { "IBKR Investor".to_string() } else { investor_name },
        dob: None,
        mobile: None,
        nominee: None,
        demat_id: None,
        landline: None,
        address: None,
        email: None,
        pan: None,
        ckyc_compliance: None,
        xfina: None,
    };

    let profile = EquityProfile {
        holders: EquityHolders {
            holder: vec![holder],
        }
    };

    let summary = EquitySummary {
        investment: EquityInvestment {
            holdings: EquityHoldings {
                r#type: None,
                holding: final_holdings,
            }
        },
        investment_value: total_investment_value,
        current_value: total_current_value,
        xfina: None,
    };

    let transactions = EquityTransactions {
        start_date: statement_start_date,
        end_date: statement_end_date,
        transaction: final_transactions,
        xfina: None,
    };

    let xfina_ext = XfinaEquityAccount {
        institution_name: Some("Interactive Brokers".to_string()),
        generated_date,
        date_only_paths: None,
    };

    Ok(EquityAccount { 
        r#type: EquityFiType::Equities,
        masked_acc_number: account_no,
        version: 1.1,
        linked_acc_ref: None,
        profile: Some(profile),
        summary: Some(summary),
        transactions: Some(transactions),
        xfina: Some(xfina_ext),
    })
}
