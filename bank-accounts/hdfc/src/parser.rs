use calamine::{Reader, open_workbook_auto_from_rs};
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use xfina_models::{BankAccountStatement, DepositTransaction, Holder};
use std::io::Cursor;
use regex::Regex;

pub fn parse_hdfc_xls(bytes: &[u8]) -> Result<BankAccountStatement, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Failed to open workbook: {}", e))?;
    
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("No sheets found in workbook".to_string());
    }
    
    let sheet = workbook.worksheet_range(&sheet_names[0])
        .ok_or("Sheet not found")?
        .map_err(|e| format!("Error reading sheet: {}", e))?;

    let mut stmt = BankAccountStatement::default();
    stmt.statement.institution_name = Some("HDFC Bank".to_string());

    let mut in_transactions = false;
    let mut in_summary = false;
    let mut parsed_summary_opening: Option<f64> = None;
    let mut parsed_summary_closing: Option<f64> = None;
    let mut parsed_summary_debits: Option<f64> = None;
    let mut parsed_summary_credits: Option<f64> = None;

    let re_dates = Regex::new(r"Statement From\s*:\s*(\d{2}/\d{2}/\d{4})\s*To\s*:\s*(\d{2}/\d{2}/\d{4})").unwrap();
    let mut name = String::new();

    for (row_idx, row) in sheet.rows().enumerate() {
        let row_vec: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        if row_vec.is_empty() {
            continue;
        }

        // Extract Customer Name
        if row_idx == 5 && !row_vec[0].is_empty() {
            name = row_vec[0].replace("MR", "").replace("MS", "").trim().to_string();
        }

        // Extract Account Number
        if row_idx == 14 && row_vec.len() > 4 {
            let acct_str = &row_vec[4];
            if acct_str.contains("Account No :") {
                let parts: Vec<&str> = acct_str.split(':').collect();
                if parts.len() > 1 {
                    stmt.statement.account_number = Some(parts[1].trim().split_whitespace().next().unwrap_or("").to_string());
                }
            }
        }

        // Extract Dates
        if row_idx == 15 && !row_vec[0].is_empty() {
            if let Some(caps) = re_dates.captures(&row_vec[0]) {
                if let Some(start) = caps.get(1) {
                    stmt.statement.start_date = Some(parse_date(start.as_str()));
                }
                if let Some(end) = caps.get(2) {
                    stmt.statement.end_date = Some(parse_date(end.as_str()));
                }
            }
        }

        // We are in transactions. A row of asterisks might appear first.
        if row_vec[0].starts_with('*') {
            continue;
        }

        if row_vec[0].is_empty() || row_vec[0].contains("**Continue**") {
            if row_vec[0].contains("**Continue**") || row_vec[0].contains("HDFC BANK Ltd.") {
                in_transactions = false; // pause transactions until we see "Date" header again
            }
            continue;
        }

        if row_vec[0].contains("STATEMENT SUMMARY") {
            in_transactions = false;
            in_summary = true;
            continue;
        }

        if in_summary {
            if let Ok(ob) = row_vec[0].trim().parse::<f64>() {
                parsed_summary_opening = Some(ob);
                if row_vec.len() >= 7 {
                    parsed_summary_debits = row_vec[4].trim().parse::<f64>().ok();
                    parsed_summary_credits = row_vec[5].trim().parse::<f64>().ok();
                    parsed_summary_closing = row_vec[6].trim().parse::<f64>().ok();
                }
                in_summary = false; // we got the values
            }
            continue;
        }

        if row_vec[0].contains("Generated On:") {
            if row_vec.len() > 1 {
                let gen_str = row_vec[1].trim();
                if let Ok(dt) = NaiveDateTime::parse_from_str(gen_str, "%d-%b-%Y %H:%M:%S") {
                    stmt.statement.generated_date = Some(Utc.from_utc_datetime(&dt));
                } else if let Ok(d) = NaiveDate::parse_from_str(gen_str, "%d-%b-%Y") {
                    stmt.statement.generated_date = Some(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()));
                }
            }
            continue;
        }

        if !in_transactions && !in_summary {
            if row_vec[0] == "Date" && row_vec.len() > 6 && row_vec[1] == "Narration" {
                in_transactions = true;
            }
            continue;
        }

        if row_vec[0].contains("Opening Balance") {
            continue;
        }

        // Parse a transaction line
        if row_vec.len() >= 7 && in_transactions {
            let date_str = row_vec[0].trim();
            if date_str.is_empty() || date_str.starts_with('*') || date_str.len() < 8 {
                continue; // Skip lines that aren't transactions
            }

            let date = parse_date(date_str);
            let description = row_vec[1].trim().to_string();
            let ref_no = row_vec[2].trim().to_string();
            let value_date_str = row_vec[3].trim();
            let value_date = if !value_date_str.is_empty() {
                Some(parse_date(value_date_str))
            } else {
                None
            };

            let withdrawal_str = row_vec[4].trim().replace(",", "");
            let deposit_str = row_vec[5].trim().replace(",", "");
            let balance_str = row_vec[6].trim().replace(",", "");

            let (tx_type, amount) = if !withdrawal_str.is_empty() {
                ("DEBIT".to_string(), withdrawal_str.parse::<f64>().unwrap_or(0.0))
            } else if !deposit_str.is_empty() {
                ("CREDIT".to_string(), deposit_str.parse::<f64>().unwrap_or(0.0))
            } else {
                continue;
            };

            let balance = balance_str.parse::<f64>().ok();

            if stmt.summary.opening_balance.is_none() {
                // If opening balance isn't set, infer it from the first transaction's balance and amount
                if let Some(bal) = balance {
                    if tx_type == "CREDIT" {
                        stmt.summary.opening_balance = Some(bal - amount);
                    } else {
                        stmt.summary.opening_balance = Some(bal + amount);
                    }
                }
            }

            stmt.summary.current_balance = balance.unwrap_or(0.0); // Updates with each transaction so it holds the final one

            let mode = if description.starts_with("UPI-") || description.contains("UPI") {
                Some("UPI".to_string())
            } else if description.starts_with("IB FUNDS TRANSFER") {
                Some("FT".to_string())
            } else if description.starts_with("IB BILLPAY") {
                Some("BILLPAY".to_string())
            } else if description.starts_with("POS ") || description.contains("CCPAY") {
                Some("CARD".to_string())
            } else if description.contains("ATM") || description.contains("CASH") {
                Some("CASH".to_string())
            } else {
                None
            };

            stmt.transactions.push(DepositTransaction {
                txn_id: None,
                date,
                value_date,
                narration: description.to_string(),
                reference: if ref_no.is_empty() { None } else { Some(ref_no.to_string()) },
                mode,
                r#type: tx_type,
                amount,
                current_balance: balance.unwrap_or(0.0),
            });
        }
    }

    let mut holder = Holder::default();
    holder.name = name;
    stmt.profile.holders.holder.push(holder);

    let total_debits = stmt.total_debits();
    let total_credits = stmt.total_credits();
    
    // Validation against summary
    let tolerance = 0.05; // 5 paise tolerance
    
    if let Some(expected) = parsed_summary_debits {
        if (total_debits - expected).abs() > tolerance {
            return Err(format!("Total debits validation failed: computed {}, expected {}", total_debits, expected));
        }
    }
    
    if let Some(expected) = parsed_summary_credits {
        if (total_credits - expected).abs() > tolerance {
            return Err(format!("Total credits validation failed: computed {}, expected {}", total_credits, expected));
        }
    }
    
    if let (Some(expected), Some(computed)) = (parsed_summary_opening, stmt.summary.opening_balance) {
        if (computed - expected).abs() > tolerance {
            return Err(format!("Opening balance validation failed: computed {}, expected {}", computed, expected));
        }
    }
    
    if let (Some(expected), computed) = (parsed_summary_closing, stmt.summary.current_balance) {
        if (expected - computed).abs() > 0.01 {
            return Err(format!("Closing balance validation failed: computed {}, expected {}", computed, expected));
        }
    }

    Ok(stmt)
}

fn parse_date(date_str: &str) -> NaiveDate {
    // Try DD/MM/YY
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%y") {
        return parsed;
    }
    // Try DD/MM/YYYY
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        return parsed;
    }
    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
}
