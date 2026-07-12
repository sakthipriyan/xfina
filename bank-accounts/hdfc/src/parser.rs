use calamine::{Reader, open_workbook_auto_from_rs};
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc, FixedOffset};
use std::io::Cursor;
use rust_decimal::Decimal;
use xfina_models::deposit::{DepositAccount, Transaction, XfinaDepositAccount, XfinaTransactions, XfinaSummary, Profile, Holders, Holder, Summary, Transactions, HoldersType, TransactionType, TransactionMode, FiType, StatusTypes, HoldingNominee};
use xfina_models::mask_account_number;
use regex::Regex;

pub fn parse_hdfc_xls(bytes: &[u8]) -> Result<DepositAccount, String> {
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

    let mut stmt = DepositAccount::default();
    stmt.r#type = FiType::Deposit;
    stmt.version = 1.1;
    
    let mut xfina_account = XfinaDepositAccount::default();
    xfina_account.institution_name = Some("HDFC Bank".to_string());
    
    let mut date_only_paths = Vec::new();

    let mut in_transactions = false;
    let mut in_summary = false;
    let mut parsed_summary_opening: Option<Decimal> = None;
    let mut parsed_summary_closing: Option<Decimal> = None;
    let mut parsed_summary_debits: Option<Decimal> = None;
    let mut parsed_summary_credits: Option<Decimal> = None;

    let mut parsed_transactions = Vec::new();
    let mut transactions_obj = Transactions::default();
    let mut summary = Summary::default();
    let mut holder = Holder::default();

    // Regexes for header extraction
    let re_nomination = Regex::new(r"Nomination\s*:\s*(Registered|Not[-\s]Registered)").unwrap();
    let re_dates = Regex::new(r"Statement From\s*:\s*(\d{2}/\d{2}/\d{4})\s*To\s*:\s*(\d{2}/\d{2}/\d{4})").unwrap();
    let re_branch = Regex::new(r"(?:Account Branch|Branch)\s*:\s*([^|]+)").unwrap();
    let re_od_limit = Regex::new(r"OD Limit\s*:\s*([\d\.]+)").unwrap();
    let re_currency = Regex::new(r"Currency\s*:\s*([A-Za-z]+)").unwrap();
    let re_ifsc = Regex::new(r"(?:RTGS/NEFT IFSC|IFSC)\s*:\s*([A-Z0-9]+)").unwrap();
    let re_micr = Regex::new(r"MICR\s*:\s*(\d+)").unwrap();
    let re_open_date = Regex::new(r"A/C Open Date\s*:\s*(\d{2}/\d{2}/\d{4})").unwrap();
    let re_account_no = Regex::new(r"Account No\s*:\s*([\w]+)").unwrap();
    let re_cust_id = Regex::new(r"Cust ID\s*:\s*(\d+)").unwrap();
    let re_account_product = Regex::new(r"Account No\s*:\s*[\w]+\s+([^|]+)").unwrap();

    let mut header_text = String::new();
    let mut address_lines = Vec::new();
    let mut name = String::new();

    for (row_idx, row) in sheet.rows().enumerate() {
        let row_vec: Vec<String> = row.iter().map(|c| c.to_string().replace("\u{0}", "").trim().to_string()).collect();
        if row_vec.is_empty() || row_vec.iter().all(|s| s.is_empty()) {
            continue;
        }

        // Header parsing logic (first 20 rows)
        if row_idx < 20 {
            for cell in &row_vec {
                if !cell.is_empty() {
                    header_text.push_str(cell);
                    header_text.push_str(" | ");
                }
            }

            // Customer Name
            if row_idx == 5 && !row_vec[0].is_empty() {
                name = row_vec[0].replace("MR", "").replace("MS", "").trim().to_string();
            }

            // Address logic: usually left side from rows 5 to 10
            if row_idx >= 5 && row_idx <= 10 {
                let col0 = row_vec[0].trim();
                if !col0.is_empty() && !col0.contains("JOINT HOLDERS") && !col0.contains("Nomination") && !col0.contains("MR") && !col0.contains("MS") {
                    address_lines.push(col0.to_string());
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
            if let Ok(ob) = row_vec[0].trim().replace(",", "").parse::<Decimal>() {
                parsed_summary_opening = Some(ob);
                if row_vec.len() >= 7 {
                    parsed_summary_debits = row_vec[4].trim().replace(",", "").parse::<Decimal>().ok();
                    parsed_summary_credits = row_vec[5].trim().replace(",", "").parse::<Decimal>().ok();
                    parsed_summary_closing = row_vec[6].trim().replace(",", "").parse::<Decimal>().ok();
                }
                in_summary = false; // we got the values
            }
            continue;
        }

        if row_vec[0].contains("Generated On:") {
            if row_vec.len() > 1 {
                let gen_str = row_vec[1].trim();
                let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                if let Ok(dt) = NaiveDateTime::parse_from_str(gen_str, "%d-%b-%Y %H:%M:%S") {
                    xfina_account.generated_date = ist_offset.from_local_datetime(&dt).single().map(|d| d.with_timezone(&Utc));
                } else if let Ok(d) = NaiveDate::parse_from_str(gen_str, "%d-%b-%Y") {
                    xfina_account.generated_date = ist_offset.from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap()).single().map(|d| d.with_timezone(&Utc));
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

            let withdrawal: Decimal = withdrawal_str.parse().unwrap_or(Decimal::from(0));
            let deposit: Decimal = deposit_str.parse().unwrap_or(Decimal::from(0));

            let (tx_type, amount) = if withdrawal > Decimal::from(0) {
                (TransactionType::Debit, withdrawal)
            } else if deposit > Decimal::from(0) {
                (TransactionType::Credit, deposit)
            } else {
                continue;
            };

            let balance: Decimal = balance_str.parse().unwrap_or(Decimal::from(0));

            let mode = if description.starts_with("UPI-") || description.contains("UPI") {
                Some(TransactionMode::Upi)
            } else if description.starts_with("POS ") || description.contains("CCPAY") {
                Some(TransactionMode::Card)
            } else if description.contains("ATM") || description.contains("CASH") {
                Some(TransactionMode::Cash)
            } else {
                None
            };

            // Convert IST to UTC for transactions
            let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
            let tx_dt = date.and_hms_opt(0, 0, 0).unwrap();
            let utc_tx_dt = ist_offset.from_local_datetime(&tx_dt).single().map(|d| d.with_timezone(&Utc));

            parsed_transactions.push(Transaction {
                txn_id: None,
                transaction_timestamp: utc_tx_dt,
                value_date: value_date,
                narration: description.to_string(),
                reference: if ref_no.is_empty() { None } else { Some(ref_no.to_string()) },
                mode,
                r#type: tx_type,
                amount,
                current_balance: balance,
            });
            
            if !date_only_paths.contains(&"transactions.transaction.transactionTimestamp".to_string()) {
                date_only_paths.push("transactions.transaction.transactionTimestamp".to_string());
            }
        }
    }

    // Process extracted header variables
    if let Some(caps) = re_nomination.captures(&header_text) {
        let value = caps[1].to_uppercase();
        if value == "REGISTERED" {
            holder.nominee = Some(HoldingNominee::Registered);
        } else if value == "NOT REGISTERED" || value.contains("NOT") {
            holder.nominee = Some(HoldingNominee::NotRegistered);
        }
    }
    
    if let Some(caps) = re_dates.captures(&header_text) {
        transactions_obj.start_date = Some(parse_date(&caps[1]));
        transactions_obj.end_date = Some(parse_date(&caps[2]));
    }
    
    if let Some(caps) = re_branch.captures(&header_text) {
        summary.branch = Some(caps[1].trim().to_string());
    }

    if let Some(caps) = re_od_limit.captures(&header_text) {
        if let Ok(limit) = caps[1].parse::<Decimal>() {
            summary.current_od_limit = Some(limit);
        }
    }
    
    if let Some(caps) = re_currency.captures(&header_text) {
        summary.currency = Some(caps[1].trim().to_string());
    }
    
    if let Some(caps) = re_ifsc.captures(&header_text) {
        summary.ifsc_code = Some(caps[1].trim().to_string());
    }
    
    if let Some(caps) = re_micr.captures(&header_text) {
        summary.micr_code = Some(caps[1].trim().to_string());
    }
    
    if let Some(caps) = re_open_date.captures(&header_text) {
        summary.opening_date = Some(parse_date(&caps[1]));
    }
    
    if let Some(caps) = re_account_no.captures(&header_text) {
        stmt.masked_acc_number = mask_account_number(caps[1].trim());
    }

    if let Some(caps) = re_cust_id.captures(&header_text) {
        holder.xfina = Some(xfina_models::deposit::XfinaHolder {
            customer_id: Some(caps[1].trim().to_string()),
        });
    }

    if !address_lines.is_empty() {
        holder.address = Some(address_lines.join(", "));
    }
    holder.name = name;

    // Statement Summary validation and integration
    let calc_debits: Decimal = parsed_transactions.iter().filter(|t| t.r#type == TransactionType::Debit).map(|t| t.amount).sum();
    let calc_credits: Decimal = parsed_transactions.iter().filter(|t| t.r#type == TransactionType::Credit).map(|t| t.amount).sum();
    
    if let Some(sd) = parsed_summary_debits {
        if calc_debits != sd {
            return Err(format!("Total debits mismatch: expected {}, got {}", sd, calc_debits));
        }
    }
    if let Some(sc) = parsed_summary_credits {
        if calc_credits != sc {
            return Err(format!("Total credits mismatch: expected {}, got {}", sc, calc_credits));
        }
    }
    
    let mut account_product: Option<String> = None;
    if let Some(caps) = re_account_product.captures(&header_text) {
        let product = caps[1].trim().to_string();
        if !product.is_empty() {
            account_product = Some(product);
        }
    }

    let mut ob_val = None;
    if let Some(ob) = parsed_summary_opening {
        ob_val = Some(ob);
    } else if let Some(first) = parsed_transactions.first() {
        ob_val = Some(if first.r#type == TransactionType::Credit { first.current_balance - first.amount } else { first.current_balance + first.amount });
    }
    
    let mut xfina_sum = XfinaSummary::default();
    xfina_sum.opening_balance = ob_val;
    xfina_sum.account_product = account_product;
    
    if xfina_sum.opening_balance.is_some() || xfina_sum.account_product.is_some() {
        summary.xfina = Some(xfina_sum);
    }

    if let Some(cb) = parsed_summary_closing {
        summary.current_balance = cb;
        if let Some(last) = parsed_transactions.last() {
            if summary.current_balance != last.current_balance {
                return Err(format!("Closing balance mismatch: expected {}, got {}", summary.current_balance, last.current_balance));
            }
        }
    } else if let Some(last) = parsed_transactions.last() {
        summary.current_balance = last.current_balance;
    }
    
    transactions_obj.transaction = parsed_transactions;
    
    let holders_list = vec![holder];
    let mut profile = Profile::default();
    let mut holders = Holders::default();
    holders.r#type = HoldersType::Single;
    holders.holder = holders_list;
    
    profile.holders = holders;
    
    stmt.profile = Some(profile);
    stmt.summary = Some(summary);
    stmt.transactions = Some(transactions_obj);
    
    if !date_only_paths.is_empty() {
        xfina_account.date_only_paths = Some(date_only_paths);
    }
    stmt.xfina = Some(xfina_account);

    Ok(stmt)
}

fn parse_date(date_str: &str) -> NaiveDate {
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%y") {
        return parsed;
    }
    if let Ok(parsed) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        return parsed;
    }
    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
}
