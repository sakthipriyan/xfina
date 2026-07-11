use rust_decimal::Decimal;
use xfina_models::deposit::{DepositAccount, Transaction, XfinaDepositAccount, XfinaTransactions, XfinaSummary, Profile, Holders, Holder, Summary, Transactions, HoldersType, TransactionType, TransactionMode, FiType, HoldingNominee};
use xfina_models::mask_account_number;
use crate::{pdf_parser, layout};
use regex::Regex;
use chrono::{NaiveDate, TimeZone, Utc};

pub fn parse_sbi_bank_statement(bytes: &[u8], password: Option<&str>, filename: Option<&str>) -> Result<DepositAccount, String> {
    let pages = pdf_parser::extract_spatial_pages(bytes, password)?;
    
    let mut statement = DepositAccount::default();
    statement.r#type = FiType::Deposit;
    statement.version = 1.1;
    
    let mut xfina_account = XfinaDepositAccount::default();
    xfina_account.institution_name = Some("SBI".to_string());

    let mut account_number = String::new();
    let mut account_name = String::new();
    let mut branch_name = String::new();
    let mut ifsc_code = String::new();
    let mut micr_code = String::new();
    let mut product = String::new();
    let mut customer_id = String::new();
    let mut currency = String::new();
    let mut nominee = String::new();
    let mut opening_date: Option<NaiveDate> = None;
    let mut address_lines: Vec<String> = Vec::new();
    let mut in_address = false;
    
    let mut date_only_paths = Vec::new();

    if let Some(fname) = filename {
        // e.g. AccountStatement_05072026_211225.pdf
        let re = Regex::new(r"(\d{2})(\d{2})(\d{4})_(\d{2})(\d{2})(\d{2})").unwrap();
        if let Some(caps) = re.captures(fname) {
            let day = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
            let month = caps.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let year = caps.get(3).unwrap().as_str().parse::<i32>().unwrap();
            let hour = caps.get(4).unwrap().as_str().parse::<u32>().unwrap();
            let min = caps.get(5).unwrap().as_str().parse::<u32>().unwrap();
            let sec = caps.get(6).unwrap().as_str().parse::<u32>().unwrap();
            if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                let dt = d.and_hms_opt(0, 0, 0).unwrap();
                let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                xfina_account.generated_date = chrono::TimeZone::from_local_datetime(&ist_offset, &dt).single().map(|dt| dt.with_timezone(&Utc));
                if !date_only_paths.contains(&"generatedDate".to_string()) {
                    date_only_paths.push("generatedDate".to_string());
                }
            }
        }
    }
    
    let date_re = Regex::new(r"^(\d{2}/\d{2}/\d{4})\s+(\d{2}/\d{2}/\d{4})").unwrap();
    let gen_date_re = Regex::new(r"Date of Statement\s*:\s*(\d{2}-\d{2}-\d{4})").unwrap();
    let stmt_from_re = Regex::new(r"(?i)Statement From\s*:\s*(\d{2}-\d{2}-\d{4})\s*to\s*(\d{2}-\d{2}-\d{4})").unwrap();
    let summary_re = Regex::new(r"^([\d,.]+C?R?D?R?)\s+\d+\s+\d+\s+([\d,.]+)\s+([\d,.]+)\s+([\d,.]+C?R?D?R?)$").unwrap();

    let mut inside_table = false;
    let mut parsed_transactions = Vec::new();
    let mut summary = Summary::default();
    let mut transactions_obj = Transactions::default();
    
    let parse_amount = |s: &str| -> Option<Decimal> {
        s.replace(",", "").replace("CR", "").replace("DR", "").parse().ok()
    };
    
    struct DescPart {
        y: f64,
        text: String,
    }

    for page in pages {
        let lines = layout::group_into_lines(&page, 2.0);
        
        // First, group lines into vertical blocks based on a y-gap threshold
        let mut blocks: Vec<Vec<&layout::Line>> = Vec::new();
        let mut current_block = Vec::new();
        let mut last_y: Option<f64> = None;
        
        for line in &lines {
            let y = line.chars.first().map(|c| c.y0).unwrap_or(0.0);
            if let Some(ly) = last_y {
                if (y - ly).abs() > 11.5 {
                    if !current_block.is_empty() {
                        blocks.push(current_block);
                        current_block = Vec::new();
                    }
                }
            }
            current_block.push(line);
            last_y = Some(y);
        }
        if !current_block.is_empty() {
            blocks.push(current_block);
        }
        
        for block in blocks {
            let mut is_header_or_footer = false;
            
            // Check for Account Number / Name in the block
            for line in &block {
                let text = line.text.trim();
                
                if let Some(caps) = gen_date_re.captures(text) {
                    if xfina_account.generated_date.is_none() {
                        if let Ok(parsed) = NaiveDate::parse_from_str(caps.get(1).unwrap().as_str(), "%d-%m-%Y") {
                            let dt = parsed.and_hms_opt(0, 0, 0).unwrap();
                            let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                            xfina_account.generated_date = chrono::TimeZone::from_local_datetime(&ist_offset, &dt).single().map(|dt| dt.with_timezone(&Utc));
                            if !date_only_paths.contains(&"generatedDate".to_string()) {
                    date_only_paths.push("generatedDate".to_string());
                }
                        }
                    }
                }
                if let Some(caps) = stmt_from_re.captures(text) {
                    if let Ok(parsed) = NaiveDate::parse_from_str(caps.get(1).unwrap().as_str(), "%d-%m-%Y") {
                        transactions_obj.start_date = Some(parsed);
                    }
                    if let Ok(parsed) = NaiveDate::parse_from_str(caps.get(2).unwrap().as_str(), "%d-%m-%Y") {
                        transactions_obj.end_date = Some(parsed);
                    }
                }
                
                if text.contains("Brought Forward") && text.contains("Total Debits") {
                    // Header line, the next block might contain the values or this block does
                }
                
                if text.len() > 30 && (text.contains("CR") || text.contains("DR")) {
                    if let Some(caps) = summary_re.captures(text) {
                        if let Some(ob) = parse_amount(caps.get(1).unwrap().as_str()) {
                            summary.xfina = Some(XfinaSummary { opening_balance: Some(ob), ..Default::default() });
                        }
                        if let Some(cb) = parse_amount(caps.get(4).unwrap().as_str()) {
                            summary.current_balance = cb;
                        }
                    }
                }

                if text.contains("Account Number") {
                    let parts: Vec<&str> = text.split(':').collect();
                    if parts.len() > 1 {
                        let acc_part = parts[1].trim().split('(').next().unwrap_or("").trim();
                        if !acc_part.is_empty() {
                            account_number = acc_part.to_string();
                        }
                    }
                    is_header_or_footer = true;
                }
                
                let x0 = line.chars.first().map(|c| c.x0).unwrap_or(0.0);
                
                if text.starts_with("Branch Name :") {
                    branch_name = text.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if text.starts_with("CIF Number :") || text.contains("CIF Number :") {
                    if let Some(idx) = text.find("CIF Number :") {
                        customer_id = text[idx + 12..].trim().to_string();
                    }
                } else if text.starts_with("Product :") {
                    product = text.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if text.starts_with("IFSC Code :") {
                    ifsc_code = text.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if text.starts_with("Currency :") {
                    currency = text.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if text.starts_with("MICR Code :") || text.contains("MICR Code :") {
                    if let Some(idx) = text.find("MICR Code :") {
                        micr_code = text[idx + 11..].trim().to_string();
                    }
                } else if text.starts_with("Nominee Name :") {
                    nominee = text.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if text.starts_with("Account open Date :") || text.contains("Account open Date :") {
                    if let Some(idx) = text.find("Account open Date :") {
                        let d_str = text[idx + 19..].trim();
                        if let Ok(parsed) = NaiveDate::parse_from_str(d_str, "%d/%m/%Y") {
                            opening_date = Some(parsed);
                        }
                    }
                }
                
                if text.starts_with("Date of Statement") || text.starts_with("Clear Balance") || text.starts_with("Branch Code :") {
                    in_address = false;
                }
                
                if text.starts_with("Mr.") || text.starts_with("Mrs.") {
                    if account_name.is_empty() && x0 < 300.0 {
                        account_name = text.to_string();
                        // Assume customer address follows name on the left side
                        in_address = true;
                    }
                } else if in_address && x0 < 300.0 && !text.starts_with("Not Available") {
                    if !text.is_empty() {
                        address_lines.push(text.to_string());
                    }
                }
                if text.contains("Balance") && text.len() < 10 {
                    inside_table = true;
                    is_header_or_footer = true;
                }
                if text.contains("Statement Summary") || text.contains("Closing Balance") {
                    inside_table = false;
                    is_header_or_footer = true;
                }
                if text.contains("Page no.") {
                    is_header_or_footer = true;
                }
            }
            
            if is_header_or_footer || !inside_table {
                continue;
            }
            
            let mut desc_parts = Vec::new();
            let mut date_str = String::new();
            let mut val_date_str = String::new();
            let mut debit = None;
            let mut credit = None;
            let mut balance = None;
            
            let parse_amt = |s: &str| -> Option<Decimal> {
                if s == "-" { return None; }
                s.replace(",", "").replace("CR", "").replace("DR", "").parse().ok()
            };
            
            for line in &block {
                let text = line.text.trim();
                if text.is_empty() { continue; }
                let min_x = line.chars.first().map(|c| c.x0).unwrap_or(0.0);
                let min_y = line.chars.first().map(|c| c.y0).unwrap_or(0.0);
                
                if let Some(caps) = date_re.captures(text) {
                    date_str = caps.get(1).unwrap().as_str().to_string();
                    val_date_str = caps.get(2).unwrap().as_str().to_string();
                    
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    if parts.len() >= 6 {
                        let len = parts.len();
                        balance = parse_amount(parts[len - 1]);
                        credit = parse_amount(parts[len - 2]);
                        debit = parse_amount(parts[len - 3]);
                        
                        let middle = parts[2..len - 3].join(" ");
                        if middle != "-" && !middle.is_empty() {
                            let clean_middle = middle.trim_end_matches('-').trim().to_string();
                            if !clean_middle.is_empty() {
                                desc_parts.push(DescPart { y: min_y, text: clean_middle });
                            }
                        }
                    }
                } else if min_x > 120.0 && min_x < 150.0 {
                    desc_parts.push(DescPart { y: min_y, text: text.to_string() });
                }
            }
            
            if !date_str.is_empty() {
                desc_parts.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
                let narration = desc_parts.into_iter().map(|d| d.text).collect::<Vec<_>>().join(" ");
                
                let mut tx_type = TransactionType::Debit;
                let mut amount = Decimal::from(0);
                
                if let Some(c) = credit {
                    tx_type = TransactionType::Credit;
                    amount = c;
                } else if let Some(d) = debit {
                    tx_type = TransactionType::Debit;
                    amount = d;
                }
                
                let date = NaiveDate::parse_from_str(&date_str, "%d/%m/%Y").unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
                let value_date = NaiveDate::parse_from_str(&val_date_str, "%d/%m/%Y").ok();
                
                let mode = if narration.starts_with("UPI/") || narration.starts_with("TRANSFER TO UPI/") || narration.starts_with("TRANSFER FROM UPI/") {
                    Some(TransactionMode::Upi)
                } else if narration.contains("NEFT") {
                    Some(TransactionMode::Ft)
                } else if narration.contains("IMPS") {
                    Some(TransactionMode::Ft)
                } else if narration.contains("ATM") || narration.contains("CASH") {
                    Some(TransactionMode::Cash)
                } else {
                    None
                };

                let tx = Transaction {
                    txn_id: None,
                    transaction_timestamp: Some(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())),
                    value_date,
                    narration,
                    reference: None,
                    r#type: tx_type,
                    amount,
                    current_balance: balance.unwrap_or(Decimal::from(0)),
                    mode,
                };
                parsed_transactions.push(tx);
                
                if !date_only_paths.contains(&"transactions.transaction.transactionTimestamp".to_string()) {
                    date_only_paths.push("transactions.transaction.transactionTimestamp".to_string());
                }
            }
        }
    }
    
    if let Some(first) = parsed_transactions.first() {
        let ob = if first.r#type == TransactionType::Credit { first.current_balance - first.amount } else { first.current_balance + first.amount };
        summary.xfina = Some(XfinaSummary { opening_balance: Some(ob), ..Default::default() });
    }
    if let Some(last) = parsed_transactions.last() {
        summary.current_balance = last.current_balance;
    }
if !account_number.is_empty() {
        statement.masked_acc_number = mask_account_number(&account_number);
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
    if !currency.is_empty() {
        summary.currency = Some(currency);
    }
    if let Some(od) = opening_date {
        summary.opening_date = Some(od);
    }
    if let Some(mut xfina_summary) = summary.xfina.take() {
        if !product.is_empty() {
            xfina_summary.account_product = Some(product);
        }
        summary.xfina = Some(xfina_summary);
    } else if !product.is_empty() {
        summary.xfina = Some(XfinaSummary {
            account_product: Some(product),
            ..Default::default()
        });
    }

    let mut holder = Holder::default();
    holder.name = account_name;
    if !address_lines.is_empty() {
        holder.address = Some(address_lines.join(", "));
    }
    if !nominee.is_empty() && nominee != "Not Available" {
        holder.nominee = Some(HoldingNominee::Registered); // Usually XXXXX implies registered
    }
    
    use xfina_models::deposit::XfinaHolder;
    let mut xfina_holder = XfinaHolder::default();
    if !customer_id.is_empty() {
        xfina_holder.customer_id = Some(customer_id);
    }
    holder.xfina = Some(xfina_holder);
    
    let mut profile = Profile::default();
    profile.holders = Holders {
        r#type: HoldersType::Single, // Adjust as necessary
        holder: vec![holder],
    };

    transactions_obj.transaction = parsed_transactions;

    
    statement.profile = Some(profile);
    statement.summary = Some(summary);
    statement.transactions = Some(transactions_obj);
    if !date_only_paths.is_empty() {
        xfina_account.date_only_paths = Some(date_only_paths);
    }
    statement.xfina = Some(xfina_account);

    Ok(statement)
}
