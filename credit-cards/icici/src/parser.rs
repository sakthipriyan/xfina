use calamine::{Reader, Xlsx, open_workbook_from_rs};
use std::io::Cursor;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use xfina_models::credit_card::{
    CreditCardAccount, CcProfile, CcHolders, CcHolder, CcSummary, PastDues, RewardPointsSummary,
    CcTransactions, CcTransaction, XfinaCreditCardAccount, CcXfinaSummary, CcXfinaTransactions, CcXfinaTransaction,
};
use xfina_models::deposit::TransactionType;
use std::collections::HashMap;
use xfina_models::date_utils;
use regex::Regex;

pub fn parse_icici_statement(bytes: &[u8], filename: Option<&str>) -> Result<CreditCardAccount, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| format!("Failed to open workbook: {}", e))?;

    let sheet_names = workbook.sheet_names().to_owned();
    let first_sheet = sheet_names.first().ok_or("No sheets found in workbook")?;
    let range = workbook.worksheet_range(first_sheet)
        .map_err(|e| format!("Failed to get worksheet: {}", e))?;

    let mut stmt = CreditCardAccount::default();
    stmt.r#type = "credit_card".to_string();
    stmt.version = 1.1;

    let mut holder = CcHolder::default();
    let mut xfina_account = XfinaCreditCardAccount::default();
    xfina_account.institution_name = Some("ICICI".to_string());
    
    let mut date_only_paths = Vec::new();

    if let Some(fname) = filename {
        let re = Regex::new(r"(\d{2}-\d{2}-\d{4})").unwrap();
        if let Some(caps) = re.captures(fname) {
            if let Some(m) = caps.get(1) {
                if let Ok(d) = NaiveDate::parse_from_str(m.as_str(), "%d-%m-%Y") {
                    let dt = d.and_hms_opt(0, 0, 0).unwrap();
                    let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                    xfina_account.generated_date = chrono::TimeZone::from_local_datetime(&ist_offset, &dt).single().map(|dt| dt.with_timezone(&Utc));
                    date_only_paths.push("generatedDate".to_string());
                }
            }
        }
    }
    let mut summary = CcSummary::default();
    let mut xfina_summary = CcXfinaSummary::default();
    
    let mut transactions_list = Vec::new();
    let mut xfina_txns = CcXfinaTransactions::default();

    let mut in_transactions = false;

    let mut card_holder_name = String::new();
    let mut previous_balance = Decimal::new(0, 0);
    let mut purchases = Decimal::new(0, 0);
    let mut payments = Decimal::new(0, 0);
    let mut total_due = Decimal::new(0, 0);
    let mut min_due = Decimal::new(0, 0);

    let mut default_rewards = 0;
    let mut owner_credit_breakdown = HashMap::new();
    let mut owner_debit_breakdown = HashMap::new();

    for row in range.rows() {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        if cells.is_empty() { continue; }

        let col0 = cells.get(0).map(|s| s.trim()).unwrap_or("");

        if !in_transactions {
            for i in [0, 8] {
                if let (Some(key_str), Some(val_str)) = (cells.get(i), cells.get(i + 4)) {
                    let key = key_str.trim();
                    let val = val_str.replace("INR", "").trim().to_string();
                    
                    match key {
                        "Card Holder Name" => {
                            card_holder_name = val.clone();
                            holder.name = card_holder_name.clone();
                        }
                        "Previous Balance" => {
                            previous_balance = parse_decimal(&val).unwrap_or_default();
                        }
                        "Payments and Other Credits" => {
                            payments = parse_decimal(&val).unwrap_or_default();
                        }
                        "Purchases and Other Charges" => {
                            purchases = parse_decimal(&val).unwrap_or_default();
                        }
                        "Total Amount Due" => {
                            total_due = parse_decimal(&val).unwrap_or_default();
                            summary.total_due_amount = Some(total_due);
                        }
                        "Minimum Amount Due" => {
                            min_due = parse_decimal(&val).unwrap_or_default();
                            summary.min_due_amount = Some(min_due);
                        }
                        "Available Credit Limit" => {
                            summary.available_credit = parse_decimal(&val);
                        }
                        "Total Credit Limit" => {
                            summary.credit_limit = parse_decimal(&val);
                        }
                        "Available Cash Limit" => {
                            // in some formats, they might map it differently, we will just use cash_limit for total
                        }
                        "Total Cash Limit" => {
                            summary.cash_limit = parse_decimal(&val);
                        }
                        "Statement Date" => {
                            summary.last_statement_date = parse_date(&val);
                        }
                        "Payment Due Date" => {
                            summary.due_date = parse_date(&val);
                        }
                        "Statement Period" => {
                            if let Some((start, end)) = val.split_once(" TO ") {
                                let mut txns = CcTransactions::default();
                                txns.start_date = parse_date(start);
                                txns.end_date = parse_date(end);
                                xfina_txns.start_date_derived = Some(false);
                                xfina_txns.end_date_derived = Some(false);
                                stmt.transactions = Some(txns);
                            }
                        }
                        "Transaction Date" => {
                            in_transactions = true;
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Parse Transactions
            // 0=Date, 4=Details, 8=Amount, 12=Reward Points, 16=Ref Number
            let date_str = cells.get(0).map(|s| s.trim()).unwrap_or("");
            if date_str.is_empty() || date_str == "Transaction Date" { continue; } // skip empty or header
            
            let details = cells.get(4).map(|s| s.trim()).unwrap_or("").to_string();
            let amount_str = cells.get(8).map(|s| s.trim()).unwrap_or("").to_string();
            
            // Parse amount and type
            let is_credit = amount_str.ends_with("Cr.");
            let is_debit = amount_str.ends_with("Dr.");
            let txn_type = if is_credit { TransactionType::Credit } else { TransactionType::Debit };
            
            let amt_clean = amount_str.replace("Dr.", "").replace("Cr.", "").replace("INR", "").trim().to_string();
            let amount = parse_decimal(&amt_clean).unwrap_or_default().abs();
            
            let reward_points = cells.get(12).and_then(|s| s.trim().parse::<i32>().ok());
            
            let mut tx_xfina = CcXfinaTransaction::default();
            tx_xfina.owner = Some(card_holder_name.clone());
            tx_xfina.reward_points = reward_points;

            let mut parsed_date = parse_date(date_str);
            if parsed_date.is_none() {
                if let Some(stmt_date) = summary.last_statement_date {
                    parsed_date = parse_partial_date(date_str, stmt_date);
                }
            }
            if !date_only_paths.contains(&"transactions.transaction.txnDate".to_string()) {
                date_only_paths.push("transactions.transaction.txnDate".to_string());
                date_only_paths.push("transactions.transaction.valueDate".to_string());
            }
            transactions_list.push(CcTransaction {
                txn_date: parsed_date.clone(),
                value_date: parsed_date,
                narration: details,
                amount,
                txn_type,
                txn_id: None,
                statement_date: None,
                mcc: None,
                masked_card_number: None,
                xfina: Some(tx_xfina),
            });
            
            // Aggregations
            use rust_decimal::prelude::ToPrimitive;
            let amt_f64 = amount.to_f64().unwrap_or(0.0);
            
            if let Some(pts) = reward_points {
                default_rewards += pts;
            }
            if txn_type == TransactionType::Credit {
                *owner_credit_breakdown.entry(card_holder_name.clone()).or_insert(0.0) += amt_f64;
            } else if txn_type == TransactionType::Debit {
                *owner_debit_breakdown.entry(card_holder_name.clone()).or_insert(0.0) += amt_f64;
            }
        }
    }
    
    xfina_summary.opening_balance = Some(previous_balance);
    xfina_summary.payment_credit = Some(payments);
    xfina_summary.purchases_debits = Some(purchases);
    summary.finance_charges = Some(Decimal::new(0, 0)); // not clearly separated in this ICICI extract
    xfina_summary.owner_credit_breakdown = owner_credit_breakdown;
    xfina_summary.owner_debit_breakdown = owner_debit_breakdown;
    
    xfina_summary.reward_points_summary = Some(RewardPointsSummary {
        default_rewards,
        opening_balance: 0,
        earned: default_rewards,
        disbursed: 0,
        adjusted_lapsed: 0,
        closing_balance: 0,
        expiring_in_30_days: None,
        expiring_in_60_days: None,
    });
    
    let stmt_date_opt = summary.last_statement_date.clone();
    summary.xfina = Some(xfina_summary);
    stmt.summary = Some(summary);
    
    stmt.profile = Some(CcProfile {
        holders: CcHolders {
            holder: vec![holder],
        }
    });

    transactions_list.sort_by(|a, b| a.txn_date.cmp(&b.txn_date));

    let mut txns = stmt.transactions.unwrap_or_default();
    
    if txns.start_date.is_none() || txns.end_date.is_none() {
        if let Some(stmt_date) = stmt_date_opt {
            let (start, end) = xfina_models::date_utils::derive_statement_period(stmt_date);
            txns.start_date = Some(start);
            txns.end_date = Some(end);
            xfina_txns.start_date_derived = Some(true);
            xfina_txns.end_date_derived = Some(true);
        } else {
            if txns.start_date.is_none() {
                if let Some(first) = transactions_list.first() {
                    txns.start_date = first.txn_date.clone();
                    xfina_txns.start_date_derived = Some(true);
                }
            }
            if txns.end_date.is_none() {
                if let Some(last) = transactions_list.last() {
                    txns.end_date = last.txn_date.clone();
                    xfina_txns.end_date_derived = Some(true);
                }
            }
        }
    }
    
    txns.transaction = transactions_list;
    txns.xfina = Some(xfina_txns);
    stmt.transactions = Some(txns);
    
    if !date_only_paths.is_empty() {
        xfina_account.date_only_paths = Some(date_only_paths);
    }
    stmt.xfina = Some(xfina_account);
    
    Ok(stmt)
}

fn parse_decimal(val: &str) -> Option<Decimal> {
    let clean = val.replace(",", "");
    clean.parse::<Decimal>().ok()
}

fn parse_date(val: &str) -> Option<NaiveDate> {
    let iso = xfina_models::parse_indian_date(val);
    let s = iso.split('T').next().unwrap_or(&iso);
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn parse_partial_date(val: &str, stmt_date: NaiveDate) -> Option<NaiveDate> {
    let val = val.trim();
    let parts: Vec<&str> = if val.contains('/') {
        val.split('/').collect()
    } else {
        val.split('-').collect()
    };
    if parts.len() == 2 {
        let day = parts[0].trim().parse::<u32>().unwrap_or(0);
        let month_str = parts[1].trim();
        let month = if month_str.chars().all(|c| c.is_ascii_digit()) {
            month_str.parse::<u32>().unwrap_or(0)
        } else {
            match month_str.to_lowercase().as_str() {
                "jan" => 1,
                "feb" => 2,
                "mar" => 3,
                "apr" => 4,
                "may" => 5,
                "jun" => 6,
                "jul" => 7,
                "aug" => 8,
                "sep" => 9,
                "oct" => 10,
                "nov" => 11,
                "dec" => 12,
                _ => 0,
            }
        };
        if day > 0 && month > 0 {
            return Some(date_utils::derive_transaction_date(stmt_date, day, month));
        }
    }
    None
}
