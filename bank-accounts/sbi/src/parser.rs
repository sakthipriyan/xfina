use finx_models::{BankAccountStatement, BankTransaction};
use crate::{pdf_parser, layout};
use regex::Regex;
use chrono::NaiveDate;

pub fn parse_sbi_bank_statement(bytes: &[u8], password: Option<&str>) -> Result<BankAccountStatement, String> {
    let pages = pdf_parser::extract_spatial_pages(bytes, password)?;
    
    let mut transactions = Vec::new();
    let mut account_number = String::new();
    let mut account_name = String::new();
    
    let date_re = Regex::new(r"^(\d{2}/\d{2}/\d{4})\s+(\d{2}/\d{2}/\d{4})").unwrap();
    let num_re = Regex::new(r"^[0-9,.]+(CR|DR)?$").unwrap();

    let mut inside_table = false;
    
    // To handle description blocks for a transaction, we keep track of the last transaction pushed
    // and append descriptions to it.
    let mut last_tx_idx: Option<usize> = None;

    for page in pages {
        let lines = layout::group_into_lines(&page, 2.0);
        
        for line in lines {
            let text = line.text.trim();
            if text.is_empty() { continue; }
            let min_x = line.chars.first().map(|c| c.x0).unwrap_or(0.0);
            
            // Extract Account Number
            if text.contains("Account Number") {
                let parts: Vec<&str> = text.split(':').collect();
                if parts.len() > 1 {
                    let acc_part = parts[1].trim().split('(').next().unwrap_or("").trim();
                    if !acc_part.is_empty() {
                        account_number = acc_part.to_string();
                    }
                }
                continue;
            }
            
            if text.contains("Welcome:") {
                // Usually the next few lines contain the name, but let's just grab the Mrs. / Mr. line if it exists.
                // In SBI, the name is often right below Welcome: or beside it.
                // We'll leave account_name blank for now or parse it if needed.
            }
            if text.starts_with("Mr.") || text.starts_with("Mrs.") {
                if account_name.is_empty() {
                    account_name = text.to_string();
                }
            }

            if text.contains("Balance") && text.len() < 10 {
                // Table header hint
                inside_table = true;
                continue;
            }
            
            if text.contains("Statement Summary") || text.contains("Closing Balance") {
                inside_table = false;
            }

            if inside_table {
                // Check if line is a main transaction line (Starts with Date Date)
                if let Some(caps) = date_re.captures(text) {
                    let date_str = caps.get(1).unwrap().as_str();
                    let val_date_str = caps.get(2).unwrap().as_str();
                    
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    
                    // Format: 02/04/2026 02/04/2026 - - 47,819.00 70,318.26
                    // Usually: Date ValueDate ChqNo Debit Credit Balance
                    // But wait, the line in the dump was: "02/04/2026 02/04/2026 - - 47,819.00 70,318.26" (6 parts)
                    // If it has 6 parts and they are mostly numbers/dashes:
                    
                    let mut debit = None;
                    let mut credit = None;
                    let mut balance = None;
                    let mut ref_no = String::new();
                    
                    let parse_amt = |s: &str| -> Option<f64> {
                        if s == "-" { return None; }
                        s.replace(",", "").replace("CR", "").replace("DR", "").parse().ok()
                    };

                    if parts.len() >= 6 {
                        let len = parts.len();
                        balance = parse_amt(parts[len - 1]);
                        credit = parse_amt(parts[len - 2]);
                        debit = parse_amt(parts[len - 3]);
                        ref_no = parts[2..len - 3].join(" ");
                        if ref_no == "-" {
                            ref_no = String::new();
                        }
                    }
                    
                    let mut tx_type = "Debit".to_string();
                    let mut amount = 0.0;
                    
                    if let Some(c) = credit {
                        tx_type = "Credit".to_string();
                        amount = c;
                    } else if let Some(d) = debit {
                        tx_type = "Debit".to_string();
                        amount = d;
                    }

                    let tx = BankTransaction {
                        date: NaiveDate::parse_from_str(date_str, "%d/%m/%Y").unwrap().format("%Y-%m-%d").to_string(),
                        value_date: Some(NaiveDate::parse_from_str(val_date_str, "%d/%m/%Y").unwrap().format("%Y-%m-%d").to_string()),
                        description: String::new(),
                        reference_number: if ref_no.is_empty() { None } else { Some(ref_no) },
                        tx_type,
                        amount,
                        balance,
                    };
                    
                    transactions.push(tx);
                    last_tx_idx = Some(transactions.len() - 1);
                } else if min_x > 120.0 && min_x < 150.0 {
                    // This is a description line for the current transaction
                    if let Some(idx) = last_tx_idx {
                        let tx = &mut transactions[idx];
                        if !tx.description.is_empty() {
                            tx.description.push(' ');
                        }
                        tx.description.push_str(text);
                    }
                } else if text.contains("Page no.") || text.contains("Balance") {
                    // Ignore footer / header
                }
            }
        }
    }

    use finx_models::CustomerInfo;

    Ok(BankAccountStatement {
        bank_name: "SBI".to_string(),
        account_number: if account_number.is_empty() { None } else { Some(account_number) },
        customer_info: CustomerInfo {
            name: account_name,
            address: String::new(),
            customer_gstn: None,
        },
        statement_start_date: None,
        statement_end_date: None,
        opening_balance: None,
        closing_balance: None,
        transactions,
    })
}
