use calamine::{Reader, open_workbook_auto_from_rs};
use chrono::{NaiveDate, NaiveDateTime};
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
                    stmt.generated_date = Some(dt.format("%Y-%m-%d").to_string());
                } else if let Ok(d) = NaiveDate::parse_from_str(gen_str, "%d-%b-%Y") {
                    stmt.generated_date = Some(d.format("%Y-%m-%d").to_string());
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
            continue; // Reached the end of the statement or a header we can ignore
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

    let total_debits: f64 = stmt.transactions.iter().filter(|t| t.tx_type == "Debit").map(|t| t.amount).sum();
    let total_credits: f64 = stmt.transactions.iter().filter(|t| t.tx_type == "Credit").map(|t| t.amount).sum();
    
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
    
    if let (Some(expected), Some(computed)) = (parsed_summary_opening, stmt.opening_balance) {
        if (computed - expected).abs() > tolerance {
            return Err(format!("Opening balance validation failed: computed {}, expected {}", computed, expected));
        }
    }
    
    if let (Some(expected), Some(computed)) = (parsed_summary_closing, stmt.closing_balance) {
        if (computed - expected).abs() > tolerance {
            return Err(format!("Closing balance validation failed: computed {}, expected {}", computed, expected));
        }
    }

    if !stmt.transactions.is_empty() {
        stmt.total_debits = Some(total_debits);
        stmt.total_credits = Some(total_credits);
    }

    Ok(stmt)
}

fn parse_date(date_str: &str) -> String {
    // Try DD/MM/YY
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%y") {
        return parsed.format("%Y-%m-%d").to_string();
    }
    // Try DD/MM/YYYY
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        return parsed.format("%Y-%m-%d").to_string();
    }
    date_str.to_string()
}
