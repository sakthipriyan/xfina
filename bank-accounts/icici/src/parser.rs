use chrono::NaiveDate;
use calamine::{Reader, open_workbook_auto_from_rs};
use std::io::Cursor;
use finx_models::{BankAccountStatement, BankTransaction};
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
    statement.bank_name = "ICICI".to_string();

    if let Some(fname) = filename {
        // e.g. OpTransactionHistory05-07-2026.xls
        let re = Regex::new(r"(\d{2}-\d{2}-\d{4})").unwrap();
        if let Some(caps) = re.captures(fname) {
            if let Some(m) = caps.get(1) {
                if let Ok(d) = NaiveDate::parse_from_str(m.as_str(), "%d-%m-%Y") {
                    statement.generated_date = Some(d.format("%Y-%m-%d").to_string());
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
                statement.account_number = Some(acc_no.trim().to_string());
                statement.customer_info.name = parts.1.trim().to_string();
            }
        }

        if row_vec[0] == "Transaction Period" && row_vec.len() >= 3 {
            // Not easily parsable as strict dates if it says "Last 1 Month"
            // We'll leave statement_start_date and statement_end_date empty for now
            // or populate it based on the min/max dates in transactions.
        }

        if row_vec[0] == "S No." {
            in_transactions = true;
            continue;
        }

        if in_transactions {
            if row_vec[0].starts_with("Legends Used in Account Statement") || row_vec[0].is_empty() {
                // End of transaction block, wait, some rows might have empty S.No? 
                // Let's break if we see "Legends"
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
                // Ensure S.No is present or it's a valid row
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

                // Parse dates
                let parsed_date = NaiveDate::parse_from_str(date_str, "%d-%b-%Y")
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|_| date_str.to_string());
                    
                let parsed_value_date = NaiveDate::parse_from_str(value_date_str, "%d-%b-%Y")
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .ok();

                let withdrawal: f64 = withdrawal_str.parse().unwrap_or(0.0);
                let deposit: f64 = deposit_str.parse().unwrap_or(0.0);
                let balance: f64 = balance_str.parse().unwrap_or(0.0);

                let (tx_type, amount) = if withdrawal > 0.0 {
                    ("Debit".to_string(), withdrawal)
                } else if deposit > 0.0 {
                    ("Credit".to_string(), deposit)
                } else {
                    continue; // Zero amount transaction? Skip.
                };

                statement.transactions.push(BankTransaction {
                    date: parsed_date,
                    value_date: parsed_value_date,
                    description: desc.to_string(),
                    reference_number: if ref_num.is_empty() { None } else { Some(ref_num.to_string()) },
                    tx_type,
                    amount,
                    balance: Some(balance),
                });
            }
        }
    }
    
    // Set opening and closing balance
    if let Some(first) = statement.transactions.first() {
        if first.tx_type == "Credit" {
            statement.opening_balance = Some(first.balance.unwrap_or(0.0) - first.amount);
        } else if first.tx_type == "Debit" {
            statement.opening_balance = Some(first.balance.unwrap_or(0.0) + first.amount);
        }
    }
    
    if let Some(last) = statement.transactions.last() {
        statement.closing_balance = last.balance;
    }
    
    // Set statement period from transactions if available
    if let Some(first) = statement.transactions.first() {
        statement.statement_start_date = Some(first.date.clone());
    }
    if let Some(last) = statement.transactions.last() {
        statement.statement_end_date = Some(last.date.clone());
    }

    let total_debits: f64 = statement.transactions.iter().filter(|t| t.tx_type == "Debit").map(|t| t.amount).sum();
    let total_credits: f64 = statement.transactions.iter().filter(|t| t.tx_type == "Credit").map(|t| t.amount).sum();
    
    if !statement.transactions.is_empty() {
        statement.total_debits = Some(total_debits);
        statement.total_credits = Some(total_credits);
    }

    Ok(statement)
}
