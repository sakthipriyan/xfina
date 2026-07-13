use calamine::{Reader, open_workbook_auto_from_rs};
use chrono::{NaiveDate, TimeZone, Utc, FixedOffset};
use std::io::Cursor;
use rust_decimal::Decimal;
use xfina_models::deposit::{DepositAccount, Transaction, XfinaDepositAccount, XfinaSummary, Profile, Holders, Holder, Summary, Transactions, HoldersType, TransactionType, TransactionMode, FiType, HoldingNominee, XfinaHolder};
use xfina_models::mask_account_number;
use regex::Regex;

pub fn parse_axis_xls(bytes: &[u8], filename: Option<&str>) -> Result<DepositAccount, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Failed to open Excel workbook: {}", e))?;

    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names.first().ok_or("No sheets found")?;
    let range = workbook
        .worksheet_range(sheet_name)
        .ok_or("Sheet not found")?
        .map_err(|e| format!("Error reading worksheet: {}", e))?;

    let mut statement = DepositAccount::default();
    statement.r#type = FiType::Deposit;
    statement.version = 1.1;
    
    let mut xfina_account = XfinaDepositAccount::default();
    xfina_account.institution_name = Some("Axis Bank".to_string());

    let mut date_only_paths = Vec::new();

    if let Some(fname) = filename {
        // e.g. Axis Bank Statement_XLS.xls - maybe a date is there somewhere, we just check
        let re = Regex::new(r"(\d{2}-\d{2}-\d{4})").unwrap();
        if let Some(caps) = re.captures(fname) {
            if let Some(m) = caps.get(1) {
                if let Ok(d) = NaiveDate::parse_from_str(m.as_str(), "%d-%m-%Y") {
                    let dt = d.and_hms_opt(0, 0, 0).unwrap();
                    let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                    xfina_account.generated_date = ist_offset.from_local_datetime(&dt).single().map(|d| d.with_timezone(&Utc));
                    date_only_paths.push("xfina.generatedDate".to_string());
                }
            }
        }
    }

    let mut in_transactions = false;
    let mut parsed_transactions: Vec<Transaction> = Vec::new();
    
    let mut account_number = String::new();
    let mut ifsc_code = String::new();
    let mut micr_code = String::new();
    let mut name = String::new();
    let mut customer_id = String::new();
    let mut nominee_reg = String::new();
    let mut mobile = String::new();
    let mut email = String::new();
    let mut pan = String::new();
    let mut is_joint = false;
    let mut address_parts = Vec::new();
    let mut summary_branch = String::new();

    let mut start_date: Option<NaiveDate> = None;
    let mut end_date: Option<NaiveDate> = None;

    for (row_idx, row) in range.rows().enumerate() {
        let row_vec: Vec<String> = row.iter().map(|c| c.to_string().trim().to_string()).collect();
        if row_vec.is_empty() {
            continue;
        }

        let first_col = row_vec[0].trim();
        
        if first_col.is_empty() && !in_transactions {
            continue;
        }

        // Parse Metadata before transactions
        if !in_transactions {
            if first_col.starts_with("Name :-") {
                if let Some(n) = first_col.strip_prefix("Name :-") {
                    name = n.trim().to_string();
                }
            } else if first_col.starts_with("Customer ID :-") {
                if let Some(cid) = first_col.strip_prefix("Customer ID :-") {
                    customer_id = cid.trim().to_string();
                }
            } else if first_col.starts_with("IFSC Code :-") {
                if let Some(ifsc) = first_col.strip_prefix("IFSC Code :-") {
                    ifsc_code = ifsc.trim().to_string();
                }
            } else if first_col.starts_with("MICR Code :-") {
                if let Some(micr) = first_col.strip_prefix("MICR Code :-") {
                    micr_code = micr.trim().to_string();
                }
            } else if first_col.starts_with("Nominee Registered :-") {
                if let Some(nom) = first_col.strip_prefix("Nominee Registered :-") {
                    nominee_reg = nom.trim().to_string();
                }
            } else if first_col.starts_with("Registered Mobile No :-") {
                if let Some(mob) = first_col.strip_prefix("Registered Mobile No :-") {
                    mobile = mob.trim().to_string();
                }
            } else if first_col.starts_with("Registered Email ID :-") {
                if let Some(eml) = first_col.strip_prefix("Registered Email ID :-") {
                    email = eml.trim().to_string();
                }
            } else if first_col.starts_with("PAN :-") {
                if let Some(p) = first_col.strip_prefix("PAN :-") {
                    pan = p.trim().to_string();
                }
            } else if first_col.starts_with("Joint Holder :-") {
                if let Some(jt) = first_col.strip_prefix("Joint Holder :-") {
                    if jt.trim() != "-" {
                        is_joint = true;
                    }
                }
            } else if first_col.starts_with("Statement of Account No") {
                // Statement of Account No - 914010032444462 for the period (From : 01-07-2026 To : 13-07-2026)
                let re_acc = Regex::new(r"Account No\s*-\s*(\d+).*?From\s*:\s*([\d-]+)\s*To\s*:\s*([\d-]+)").unwrap();
                if let Some(caps) = re_acc.captures(first_col) {
                    account_number = caps[1].to_string();
                    start_date = NaiveDate::parse_from_str(&caps[2], "%d-%m-%Y").ok();
                    end_date = NaiveDate::parse_from_str(&caps[3], "%d-%m-%Y").ok();
                }
            } else if first_col == "SRL NO" && row_vec.len() >= 8 {
                in_transactions = true;
                continue;
            } else if row_idx > 1 && row_idx < 6 && !first_col.starts_with("Joint Holder") && !first_col.is_empty() {
                // Collect address rows roughly (e.g. rows 2 to 5 typically in the sample)
                address_parts.push(first_col.to_string());
            }
        }

        if in_transactions {
            if first_col.is_empty() || first_col.starts_with("BRANCH ADDRESS") || first_col == "Legend :" {
                // End of transaction block or empty line
                if first_col.starts_with("BRANCH ADDRESS") {
                    if let Some(branch_text) = first_col.strip_prefix("BRANCH ADDRESS -") {
                        if let Some(idx) = branch_text.find(']') {
                            summary_branch = branch_text[..=idx].trim().to_string();
                        } else if let Some(idx) = branch_text.find('[') {
                            summary_branch = branch_text[..idx].trim().to_string();
                        } else if let Some(branch_name) = branch_text.split(',').next() {
                            summary_branch = branch_name.trim().to_string();
                        }
                    }
                    break;
                }
                if first_col == "Legend :" {
                    break;
                }
                continue; // Ignore empty lines inside transaction block if any
            }

            // ["1", "01-07-2026", "-", "SB:914010032444462:Int.Pd:01-04-2026 to 30-06-2026", " ", "               793.00", "            80748.07", "014"]
            if row_vec.len() >= 7 {
                let date_str = &row_vec[1];
                let ref_num = &row_vec[2];
                let desc = &row_vec[3];
                let debit_str = row_vec[4].replace(",", "");
                let credit_str = row_vec[5].replace(",", "");
                let balance_str = row_vec[6].replace(",", "");
                
                let iso_date = xfina_models::parse_indian_date(date_str);
                let parsed_date = NaiveDate::parse_from_str(&iso_date, "%Y-%m-%d").ok();
                    
                let withdrawal: Decimal = debit_str.parse().unwrap_or(Decimal::from(0));
                let deposit: Decimal = credit_str.parse().unwrap_or(Decimal::from(0));
                let balance: Decimal = balance_str.parse().unwrap_or(Decimal::from(0));
                
                if row_vec.len() >= 8 && summary_branch.is_empty() {
                    summary_branch = row_vec[7].clone();
                }

                let (tx_type, amount) = if deposit > Decimal::from(0) {
                    (TransactionType::Credit, deposit)
                } else if withdrawal > Decimal::from(0) {
                    (TransactionType::Debit, withdrawal)
                } else {
                    continue;
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

                if let Some(p_date) = parsed_date {
                    let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                    let txn_timestamp = ist_offset.from_local_datetime(&p_date.and_hms_opt(0, 0, 0).unwrap()).single().map(|dt| dt.with_timezone(&Utc));

                    parsed_transactions.push(Transaction {
                        transaction_timestamp: txn_timestamp,
                        value_date: Some(p_date),
                        narration: desc.to_string(),
                        reference: if ref_num == "-" || ref_num.is_empty() { None } else { Some(ref_num.to_string()) },
                        r#type: tx_type,
                        amount,
                        mode,
                        current_balance: balance,
                        txn_id: None,
                    });
                    
                    if !date_only_paths.contains(&"transactions.transaction.transactionTimestamp".to_string()) {
                        date_only_paths.push("transactions.transaction.transactionTimestamp".to_string());
                    }
                }
            }
        }
    }
    
    let mut summary = Summary::default();
    let mut xfina_sum = XfinaSummary::default();

    if let Some(first) = parsed_transactions.first() {
        if first.r#type == TransactionType::Credit {
            let ob = first.current_balance - first.amount;
            xfina_sum.opening_balance = Some(ob);
        } else {
            let ob = first.current_balance + first.amount;
            xfina_sum.opening_balance = Some(ob);
        }
    }
    
    if let Some(last) = parsed_transactions.last() {
        summary.current_balance = last.current_balance;
    }
    
    if !ifsc_code.is_empty() {
        summary.ifsc_code = Some(ifsc_code);
    }
    if !micr_code.is_empty() {
        summary.micr_code = Some(micr_code);
    }
    if !summary_branch.is_empty() {
        summary.branch = Some(summary_branch);
    }

    summary.xfina = Some(xfina_sum);
    
    let mut transactions_obj = Transactions::default();
    transactions_obj.start_date = start_date;
    transactions_obj.end_date = end_date;
    
    transactions_obj.transaction = parsed_transactions;

    let mut holder = Holder::default();
    holder.name = name;
    
    let address = address_parts.join(", ");
    if !address.is_empty() {
        holder.address = Some(address);
    }

    if nominee_reg.starts_with("Y") {
        holder.nominee = Some(HoldingNominee::Registered);
    } else if nominee_reg.starts_with("N") {
        holder.nominee = Some(HoldingNominee::NotRegistered);
    }

    if !mobile.is_empty() { holder.mobile = Some(mobile); }
    if !email.is_empty() { holder.email = Some(email); }
    if !pan.is_empty() { holder.pan = Some(pan); }

    let mut x_holder = XfinaHolder::default();
    if !customer_id.is_empty() {
        x_holder.customer_id = Some(customer_id);
    }
    holder.xfina = Some(x_holder);

    let mut profile = Profile::default();
    profile.holders = Holders {
        r#type: if is_joint { HoldersType::Joint } else { HoldersType::Single },
        holder: vec![holder],
    };
    
    if !account_number.is_empty() {
        statement.masked_acc_number = mask_account_number(&account_number);
    }

    statement.profile = Some(profile);
    statement.summary = Some(summary);
    statement.transactions = Some(transactions_obj);
    if !date_only_paths.is_empty() {
        xfina_account.date_only_paths = Some(date_only_paths);
    }
    statement.xfina = Some(xfina_account);

    Ok(statement)
}
