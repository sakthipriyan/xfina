use chrono::{NaiveDate, TimeZone, Utc};
use calamine::{Reader, open_workbook_auto_from_rs};
use std::io::Cursor;
use rust_decimal::Decimal;
use xfina_models::deposit::{DepositAccount, Transaction, XfinaDepositAccount, XfinaTransactions, XfinaSummary, Profile, Holders, Holder, Summary, Transactions, HoldersType, TransactionType, TransactionMode, FiType};
use xfina_models::mask_account_number;
use regex::Regex;

pub fn parse_icici_xls(bytes: &[u8], filename: Option<&str>) -> Result<DepositAccount, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Failed to open Excel workbook: {}", e))?;

    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No worksheet found")?
        .map_err(|e| format!("Error reading worksheet: {}", e))?;

    let mut statement = DepositAccount::default();
    statement.r#type = FiType::Deposit;
    statement.version = 1.1;
    
    let mut xfina_account = XfinaDepositAccount::default();
    xfina_account.institution_name = Some("ICICI".to_string());

    if let Some(fname) = filename {
        // e.g. OpTransactionHistory05-07-2026.xls
        let re = Regex::new(r"(\d{2}-\d{2}-\d{4})").unwrap();
        if let Some(caps) = re.captures(fname) {
            if let Some(m) = caps.get(1) {
                if let Ok(d) = NaiveDate::parse_from_str(m.as_str(), "%d-%m-%Y") {
                    xfina_account.generated_date = Some(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()));
                }
            }
        }
    }

    let mut in_transactions = false;
    let mut parsed_transactions = Vec::new();
    let mut holders = Vec::new();

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
                statement.masked_acc_number = mask_account_number(acc_no.trim());
                
                let mut holder = Holder::default();
                holder.name = parts.1.trim().to_string();
                holders.push(holder);
            }
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
                    
                let withdrawal: Decimal = withdrawal_str.parse().unwrap_or(Decimal::from(0));
                let deposit: Decimal = deposit_str.parse().unwrap_or(Decimal::from(0));
                let balance: Decimal = balance_str.parse().unwrap_or(Decimal::from(0));

                let (tx_type, amount) = if deposit > Decimal::from(0) {
                    (TransactionType::Credit, deposit)
                } else if withdrawal > Decimal::from(0) {
                    (TransactionType::Debit, withdrawal)
                } else {
                    continue; // Zero amount transaction? Skip.
                };

                let narration_upper = desc.to_uppercase();
                let mode = if desc.starts_with("UPI/") {
                    Some(TransactionMode::Upi)
                } else if narration_upper.contains("IMPS") {
                    Some(TransactionMode::Ft)
                } else if narration_upper.contains("NEFT") {
                    Some(TransactionMode::Ft)
                } else if desc.contains("ATM") || desc.starts_with("CASH") {
                    Some(TransactionMode::Cash)
                } else {
                    None
                };

                parsed_transactions.push(Transaction {
                    transaction_timestamp: Some(Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())),
                    value_date: Some(parsed_date),
                    narration: desc.to_string(),
                    reference: if ref_num.is_empty() { None } else { Some(ref_num.to_string()) },
                    r#type: tx_type,
                    amount,
                    mode,
                    current_balance: balance,
                    txn_id: None,
                });
            }
        }
    }
    
    let mut summary = Summary::default();

    // Set opening and closing balance
    if let Some(first) = parsed_transactions.first() {
        if first.r#type == TransactionType::Credit {
            let ob = first.current_balance - first.amount;
            summary.xfina = Some(XfinaSummary { opening_balance: Some(ob), ..Default::default() });
        } else {
            let ob = first.current_balance + first.amount;
            summary.xfina = Some(XfinaSummary { opening_balance: Some(ob), ..Default::default() });
        }
    }
    
    if let Some(last) = parsed_transactions.last() {
        summary.current_balance = last.current_balance;
    }
    
    let mut transactions_obj = Transactions::default();
    
    // Set statement period from transactions if available
    if let Some(first) = parsed_transactions.first() {
        transactions_obj.start_date = first.value_date;
    }
    if let Some(last) = parsed_transactions.last() {
        transactions_obj.end_date = last.value_date;
    }
    
    transactions_obj.transaction = parsed_transactions;
    transactions_obj.xfina = Some(XfinaTransactions { date_only: Some(true) });

    let mut profile = Profile::default();
    profile.holders = Holders {
        r#type: if holders.len() > 1 { HoldersType::Joint } else { HoldersType::Single },
        holder: holders,
    };
    
    statement.profile = Some(profile);
    statement.summary = Some(summary);
    statement.transactions = Some(transactions_obj);
    statement.xfina = Some(xfina_account);

    Ok(statement)
}
