use calamine::{Reader, open_workbook_auto_from_rs};
use chrono::NaiveDate;
use finx_models::{BankAccountStatement, BankTransaction};
use finx_models::credit_card::CustomerInfo;
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

    let mut stmt = BankAccountStatement {
        bank_name: "HDFC Bank".to_string(),
        ..Default::default()
    };

    let mut in_transactions = false;
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
                    stmt.account_number = Some(parts[1].trim().split_whitespace().next().unwrap_or("").to_string());
                }
            }
        }

        // Extract Dates
        if row_idx == 15 && !row_vec[0].is_empty() {
            if let Some(caps) = re_dates.captures(&row_vec[0]) {
                if let Some(start) = caps.get(1) {
                    stmt.statement_start_date = Some(parse_date(start.as_str()));
                }
                if let Some(end) = caps.get(2) {
                    stmt.statement_end_date = Some(parse_date(end.as_str()));
                }
            }
        }

        if !in_transactions {
            if row_vec[0] == "Date" && row_vec.len() > 6 && row_vec[1] == "Narration" {
                in_transactions = true;
            }
            continue;
        }

        // We are in transactions. A row of asterisks might appear first.
        if row_vec[0].starts_with('*') {
            continue;
        }

        if row_vec[0].is_empty() || row_vec[0].contains("**Continue**") {
            // Might be the end or a continuation, wait... HDFC has `**Continue**` and page headers in the middle of transactions for long statements!
            // Let's check for page breaks.
            if row_vec[0].contains("**Continue**") || row_vec[0].contains("HDFC BANK Ltd.") {
                in_transactions = false; // pause transactions until we see "Date" header again
            }
            continue;
        }

        if row_vec[0].contains("Statement Summary") || row_vec[0].contains("Opening Balance") || row_vec[0].contains("Generated On:") {
            break; // Reached the end of the statement
        }

        // Parse a transaction line
        if row_vec.len() >= 7 {
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
                ("Debit".to_string(), withdrawal_str.parse::<f64>().unwrap_or(0.0))
            } else if !deposit_str.is_empty() {
                ("Credit".to_string(), deposit_str.parse::<f64>().unwrap_or(0.0))
            } else {
                continue;
            };

            let balance = balance_str.parse::<f64>().ok();

            if stmt.opening_balance.is_none() {
                // If opening balance isn't set, infer it from the first transaction's balance and amount
                if let Some(bal) = balance {
                    if tx_type == "Credit" {
                        stmt.opening_balance = Some(bal - amount);
                    } else {
                        stmt.opening_balance = Some(bal + amount);
                    }
                }
            }

            stmt.closing_balance = balance; // Updates with each transaction so it holds the final one

            stmt.transactions.push(BankTransaction {
                date,
                value_date,
                description,
                reference_number: if ref_no.is_empty() { None } else { Some(ref_no) },
                tx_type,
                amount,
                balance,
            });
        }
    }

    stmt.customer_info = CustomerInfo {
        name,
        address: "".to_string(),
        customer_gstn: None,
    };

    Ok(stmt)
}

fn parse_date(date_str: &str) -> String {
    // Input format: DD/MM/YY
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%y") {
        return parsed.format("%Y-%m-%d").to_string();
    }
    date_str.to_string()
}
