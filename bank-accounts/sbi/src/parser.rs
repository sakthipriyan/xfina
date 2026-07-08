use rust_decimal::Decimal;
use xfina_models::deposit::{DepositAccount, DepositTransaction, XfinaDepositAccount, XfinaTransaction, XfinaSummary, Profile, Holders, Holder, DepositSummary, DepositTransactions, HoldershipType, TransactionType, TransactionMode, FiType};
use crate::{pdf_parser, layout};
use regex::Regex;
use chrono::{NaiveDate, TimeZone, Utc};

pub fn parse_sbi_bank_statement(bytes: &[u8], password: Option<&str>) -> Result<DepositAccount, String> {
    let pages = pdf_parser::extract_spatial_pages(bytes, password)?;
    
    let mut statement = DepositAccount::default();
    statement.r#type = FiType::Deposit;
    statement.version = 1.1;
    
    let mut xfina_account = XfinaDepositAccount::default();
    xfina_account.institution_name = Some("SBI".to_string());

    let mut account_number = String::new();
    let mut account_name = String::new();
    
    let date_re = Regex::new(r"^(\d{2}/\d{2}/\d{4})\s+(\d{2}/\d{2}/\d{4})").unwrap();
    let gen_date_re = Regex::new(r"Date of Statement\s*:\s*(\d{2}-\d{2}-\d{4})").unwrap();
    let stmt_from_re = Regex::new(r"(?i)Statement From\s*:\s*(\d{2}-\d{2}-\d{4})\s*to\s*(\d{2}-\d{2}-\d{4})").unwrap();
    let summary_re = Regex::new(r"^([\d,.]+C?R?D?R?)\s+\d+\s+\d+\s+([\d,.]+)\s+([\d,.]+)\s+([\d,.]+C?R?D?R?)$").unwrap();

    let mut inside_table = false;
    let mut parsed_transactions = Vec::new();
    let mut summary = DepositSummary::default();
    let mut transactions_obj = DepositTransactions::default();
    
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
                    if let Ok(parsed) = NaiveDate::parse_from_str(caps.get(1).unwrap().as_str(), "%d-%m-%Y") {
                        xfina_account.generated_date = Some(Utc.from_utc_datetime(&parsed.and_hms_opt(0, 0, 0).unwrap()));
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
                            summary.xfina = Some(XfinaSummary { opening_balance: Some(ob) });
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
                if text.starts_with("Mr.") || text.starts_with("Mrs.") {
                    if account_name.is_empty() {
                        account_name = text.to_string();
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
                } else if narration.starts_with("NEFT") {
                    Some(TransactionMode::Neft)
                } else if narration.starts_with("IMPS") || narration.starts_with("IMPS/") {
                    Some(TransactionMode::Imps)
                } else if narration.contains("ATM") || narration.contains("CASH") {
                    Some(TransactionMode::Cash)
                } else {
                    None
                };

                let tx = DepositTransaction {
                    txn_id: None,
                    transaction_timestamp: Some(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())),
                    value_date,
                    narration,
                    reference: None,
                    r#type: tx_type,
                    amount,
                    current_balance: balance.unwrap_or(Decimal::from(0)),
                    mode,
                    xfina: Some(XfinaTransaction {
                        posting_date: Some(date),
                    }),
                };
                parsed_transactions.push(tx);
            }
        }
    }
    
    if let Some(first) = parsed_transactions.first() {
        let ob = if first.r#type == TransactionType::Credit { first.current_balance - first.amount } else { first.current_balance + first.amount };
        summary.xfina = Some(XfinaSummary { opening_balance: Some(ob) });
    }
    if let Some(last) = parsed_transactions.last() {
        summary.current_balance = last.current_balance;
    }

    if !account_number.is_empty() {
        statement.masked_acc_number = account_number;
    }
    
    let mut holder = Holder::default();
    holder.name = account_name;
    
    let mut profile = Profile::default();
    profile.holders = Holders {
        r#type: HoldershipType::Single, // Adjust as necessary
        holder: vec![holder],
    };

    transactions_obj.transaction = parsed_transactions;
    
    statement.profile = Some(profile);
    statement.summary = Some(summary);
    statement.transactions = Some(transactions_obj);
    statement.xfina = Some(xfina_account);

    Ok(statement)
}
