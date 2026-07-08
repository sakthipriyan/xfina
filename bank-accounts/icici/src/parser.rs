use chrono::{NaiveDate, TimeZone, Utc};
use calamine::{Reader, open_workbook_auto_from_rs};
use std::io::Cursor;
use xfina_models::{BankAccountStatement, DepositTransaction, Holder};
use regex::Regex;

pub fn parse_icici_xls(bytes: &[u8], filename: Option<&str>) -> Result<BankAccountStatement, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Failed to open Excel workbook: {}", e))?;

    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No worksheet found")?
        .map_err(|e| format!("Error reading worksheet: {}", e))?;

    let mut statement = BankAccountStatement::default();
    statement.statement.institution_name = "ICICI".to_string();

    if let Some(fname) = filename {
        // e.g. OpTransactionHistory05-07-2026.xls
        let re = Regex::new(r"(\d{2}-\d{2}-\d{4})").unwrap();
        if let Some(caps) = re.captures(fname) {
            if let Some(m) = caps.get(1) {
                if let Ok(d) = NaiveDate::parse_from_str(m.as_str(), "%d-%m-%Y") {
                    statement.statement.generated_date = Some(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()));
                }
            }
        }
    }

    let mut in_transactions = false;

    for row in range.rows() {
        let row_vec: Vec<String> = row.iter().map(|c| c.to_string().trim().to_string()).collect();
        if row_vec.is_empty() {
            continue;
        }

        // Check for Metadata
        if row_vec[0] == "Account Number" && row_vec.len() >= 3 {
            let account_str = &row_vec[2];
            // Format: "055801530084 ( INR )  - SAKTHI PRIYAN H"
            if let Some(parts) = account_str.split_once(" - ") {
                let left_part = parts.0;
                let acc_no = left_part.split(' ').next().unwrap_or(left_part);
                statement.statement.account_number = Some(acc_no.trim().to_string());
                
                let mut holder = Holder::default();
                holder.name = parts.1.trim().to_string();
                statement.profile.holders.holder.push(holder);
            }
        }

        if row_vec[0] == "Transaction Period" && row_vec.len() >= 3 {
            // Not easily parsable as strict dates if it says "Last 1 Month"
        }

        if row_vec[0] == "S No." {
            in_transactions = true;
            continue;
        }

        if in_transactions {
            if row_vec[0].starts_with("Legends Used in Account Statement") || row_vec[0].is_empty() {
                // End of transaction block, break if we see "Legends"
                if row_vec[0].starts_with("Legends Used in Account Statement") {
                    break;
                }
                
                // If it's completely empty or just the first cell is empty but no other data...
                let has_data = row_vec.iter().any(|c| !c.is_empty());
                if !has_data {
                    continue;
                }
            }

            // Parse a transaction line
            // ["1", "05-Jun-2026", "05-Jun-2026", "", "NEFT-...", "0.00", "10000.00", "13217.10"]
            if row_vec.len() >= 8 {
                if row_vec[1].is_empty() {
                    continue;
                }
                
                let value_date_str = &row_vec[1];
                let date_str = &row_vec[2];
                let ref_num = &row_vec[3];
                let desc = &row_vec[4];
                let withdrawal_str = row_vec[5].replace(",", "");
                let deposit_str = row_vec[6].replace(",", "");
                let balance_str = row_vec[7].replace(",", "");
                
                if date_str.is_empty() && desc.is_empty() {
                    continue; // Skip empty rows
                }

                // Parse dates (fallback to 1970 if failure, but usually this won't happen)
                let parsed_date = NaiveDate::parse_from_str(date_str, "%d-%b-%Y")
                    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
                    
                let parsed_value_date = NaiveDate::parse_from_str(value_date_str, "%d-%b-%Y").ok();

                let withdrawal: f64 = withdrawal_str.parse().unwrap_or(0.0);
                let deposit: f64 = deposit_str.parse().unwrap_or(0.0);
                let balance: f64 = balance_str.parse().unwrap_or(0.0);

                let (tx_type, amount) = if withdrawal > 0.0 {
                    ("DEBIT".to_string(), withdrawal)
                } else if deposit > 0.0 {
                    ("CREDIT".to_string(), deposit)
                } else {
                    continue; // Zero amount transaction? Skip.
                };

                statement.transactions.push(DepositTransaction {
                    txn_id: None, // No specific txn_id provided in PDF
                    date: parsed_date,
                    value_date: parsed_value_date,
                    narration: desc.to_string(),
                    reference: if ref_num.is_empty() { None } else { Some(ref_num.to_string()) },
                    r#type: tx_type,
                    amount,
                    current_balance: Some(balance),
                });
            }
        }
    }
    
    // Set opening and closing balance
    if let Some(first) = statement.transactions.first() {
        if first.r#type == "CREDIT" {
            statement.summary.opening_balance = Some(first.current_balance.unwrap_or(0.0) - first.amount);
        } else if first.r#type == "DEBIT" {
            statement.summary.opening_balance = Some(first.current_balance.unwrap_or(0.0) + first.amount);
        }
    }
    
    if let Some(last) = statement.transactions.last() {
        statement.summary.current_balance = last.current_balance;
    }
    
    // Set statement period from transactions if available
    if let Some(first) = statement.transactions.first() {
        statement.statement.start_date = Some(first.date.clone());
    }
    if let Some(last) = statement.transactions.last() {
        statement.statement.end_date = Some(last.date.clone());
    }

    Ok(statement)
}
