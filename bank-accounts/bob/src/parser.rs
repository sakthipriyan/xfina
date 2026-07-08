use calamine::{Reader, open_workbook_auto_from_rs};
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::io::Cursor;
use rust_decimal::Decimal;
use xfina_models::deposit::{DepositAccount, DepositTransaction, XfinaDepositAccount, XfinaTransaction, XfinaSummary, Profile, Holders, Holder, DepositSummary, DepositTransactions, HoldershipType, TransactionType, TransactionMode, FiType};
use regex::Regex;

pub fn parse_bob_xls(bytes: &[u8]) -> Result<DepositAccount, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor).map_err(|e| format!("Failed to open workbook: {:?}", e))?;
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names.first().ok_or("No sheets found in workbook")?;
    let sheet = workbook.worksheet_range(sheet_name).map_err(|e| format!("Failed to read sheet: {:?}", e))?;

    let mut statement = DepositAccount::default();
    statement.r#type = FiType::Deposit;
    statement.version = 1.1;

    let mut xfina_account = XfinaDepositAccount::default();
    xfina_account.institution_name = Some("Bank of Baroda".to_string());
    
    let mut account_number = String::new();
    let mut ifsc_code = String::new();
    let mut micr_code = String::new();
    let mut branch_name = String::new();
    let mut name = String::new();
    
    let mut start_date: Option<NaiveDate> = None;
    let mut end_date: Option<NaiveDate> = None;
    let mut generated_date: Option<NaiveDate> = None;
    
    let mut parsed_transactions = Vec::new();
    let mut transactions_obj = DepositTransactions::default();
    let mut xfina_summary = XfinaSummary::default();

    let mut in_transactions = false;

    for (row_idx, row) in sheet.rows().enumerate() {
        let row_vec: Vec<String> = row.iter().map(|c| c.to_string().replace("\u{0}", "").trim().to_string()).collect();
        if row_vec.is_empty() {
            continue;
        }
        
        let first_col = row_vec[0].trim();
        
        // Extract Metadata
        if first_col.starts_with("Customer Id:") {
            if row_vec.len() > 3 {
                // Not really useful for ReBIT but good for metadata if needed
            }
            if let Some(idx) = row_vec.iter().position(|s| s.trim() == "Account No:") {
                if row_vec.len() > idx + 6 {
                    account_number = row_vec[idx + 6].trim().to_string();
                }
            }
        } else if first_col.starts_with("Branch Name:") {
            if row_vec.len() > 3 {
                branch_name = row_vec[3].trim().to_string();
            }
            if let Some(idx) = row_vec.iter().position(|s| s.trim() == "MICR Code:") {
                if row_vec.len() > idx + 6 {
                    micr_code = row_vec[idx + 6].trim().to_string();
                }
            }
        } else if first_col.starts_with("IFSC Code:") {
            if row_vec.len() > 4 {
                ifsc_code = row_vec[4].trim().to_string();
            }
        } else if first_col.starts_with("Your Account Statement as on") {
            let parts: Vec<&str> = first_col.split("as on").collect();
            if parts.len() > 1 {
                generated_date = NaiveDate::parse_from_str(parts[1].trim(), "%d/%m/%Y").ok();
            }
            if let Some(idx) = row_vec.iter().position(|s| s.starts_with("Statement Period from")) {
                let period_str = &row_vec[idx];
                let p_parts: Vec<&str> = period_str.split("from").collect();
                if p_parts.len() > 1 {
                    let d_parts: Vec<&str> = p_parts[1].split("to").collect();
                    if d_parts.len() == 2 {
                        start_date = NaiveDate::parse_from_str(d_parts[0].trim(), "%d/%m/%Y").ok();
                        end_date = NaiveDate::parse_from_str(d_parts[1].trim(), "%d/%m/%Y").ok();
                    }
                }
            }
        }
        
        // Sometimes Name is at row 12 or 1
        if row_idx == 0 && row_vec.len() > 3 && row_vec[0].is_empty() {
             if row_vec[1].contains("Holder Name") {
                 let parts: Vec<&str> = row_vec[1].split(':').collect();
                 if parts.len() > 1 {
                     name = parts[1].trim().to_string();
                 }
             }
        }
        if row_idx == 9 && first_col.len() > 5 {
             name = first_col.to_string();
        }

        // Transaction block marker
        if first_col == "TRAN DATE" {
            in_transactions = true;
            continue;
        }

        if in_transactions {
            if first_col.is_empty() {
                continue;
            }
            
            // Check for end of transactions (e.g. "This is computer-generated statement")
            if first_col.contains("This is computer-generated") || first_col.contains("Page") {
                continue;
            }
            
            // ReBIT date format: DD/MM/YYYY to ISO (YYYY-MM-DD)
            let date_str = first_col;
            let date_parts: Vec<&str> = date_str.split(' ').collect();
            let date_only = date_parts[0];
            
            let date = NaiveDate::parse_from_str(date_only, "%d/%m/%Y").ok();
            
            let val_date_str = if row_vec.len() > 2 { row_vec[2].trim() } else { "" };
            let value_date = NaiveDate::parse_from_str(val_date_str, "%d/%m/%Y").ok();
            
            let narration = if row_vec.len() > 5 { row_vec[5].trim().to_string() } else { String::new() };
            if narration.is_empty() && date.is_none() {
                continue; // Skip footer noise
            }

            let debit_str = if row_vec.len() > 11 { row_vec[11].trim() } else { "" };
            let credit_str = if row_vec.len() > 17 { row_vec[17].trim() } else { "" };
            let balance_str = if row_vec.len() > 24 { row_vec[24].trim() } else { "" };

            let parse_amt = |s: &str| -> Option<Decimal> {
                let clean = s.replace(",", "").replace("Cr", "").replace("Dr", "");
                clean.parse().ok()
            };

            let mut amount = Decimal::from(0);
            let mut tx_type = TransactionType::Debit;
            
            if let Some(cr) = parse_amt(credit_str) {
                amount = cr;
                tx_type = TransactionType::Credit;
            } else if let Some(dr) = parse_amt(debit_str) {
                amount = dr;
                tx_type = TransactionType::Debit;
            } else {
                continue; // If both empty, maybe a continuation row, but BoB usually fits in one row.
            }
            
            let mode = if narration.contains("UPI") {
                Some(TransactionMode::Upi)
            } else if narration.contains("NEFT") {
                Some(TransactionMode::Neft)
            } else if narration.contains("IMPS") {
                Some(TransactionMode::Imps)
            } else if narration.contains("CASH") {
                Some(TransactionMode::Cash)
            } else {
                None
            };
            
            let mut current_balance = Decimal::from(0);
            if let Some(bal) = parse_amt(balance_str) {
                current_balance = bal;
            }

            if let Some(dt) = date {
                let tx = DepositTransaction {
                    txn_id: None,
                    transaction_timestamp: Some(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap())),
                    value_date,
                    narration,
                    reference: None,
                    r#type: tx_type,
                    amount,
                    current_balance,
                    mode,
                    xfina: Some(XfinaTransaction {
                        posting_date: Some(dt),
                    }),
                };
                parsed_transactions.push(tx);
            }
        }
    }

    let mut summary = DepositSummary::default();
    if let Some(first) = parsed_transactions.first() {
        let ob = if first.r#type == TransactionType::Credit { first.current_balance - first.amount } else { first.current_balance + first.amount };
        summary.xfina = Some(XfinaSummary { opening_balance: Some(ob) });
    }
    if let Some(last) = parsed_transactions.last() {
        summary.current_balance = last.current_balance;
    }
    
    if !branch_name.is_empty() {
        summary.branch = Some(branch_name);
    }
    if !ifsc_code.is_empty() {
        summary.ifsc_code = Some(ifsc_code);
    }
    if !micr_code.is_empty() {
        summary.micr_code = Some(micr_code);
    }

    if !account_number.is_empty() {
        statement.masked_acc_number = account_number;
    }
    
    transactions_obj.transaction = parsed_transactions;
    transactions_obj.start_date = start_date;
    transactions_obj.end_date = end_date;
    xfina_account.generated_date = generated_date.map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()));
    
    let mut holder = Holder::default();
    holder.name = name;
    let holders = vec![holder];
    
    let mut profile = Profile::default();
    profile.holders = Holders {
        r#type: if holders.len() > 1 { HoldershipType::Joint } else { HoldershipType::Single },
        holder: holders,
    };

    statement.profile = Some(profile);
    statement.summary = Some(summary);
    statement.transactions = Some(transactions_obj);
    statement.xfina = Some(xfina_account);

    Ok(statement)
}
