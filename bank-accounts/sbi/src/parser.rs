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

    let mut inside_table = false;
    
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
            
            let parse_amt = |s: &str| -> Option<f64> {
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
                        balance = parse_amt(parts[len - 1]);
                        credit = parse_amt(parts[len - 2]);
                        debit = parse_amt(parts[len - 3]);
                        
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
                let description = desc_parts.into_iter().map(|d| d.text).collect::<Vec<_>>().join(" ");
                
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
                    date: NaiveDate::parse_from_str(&date_str, "%d/%m/%Y").unwrap().format("%Y-%m-%d").to_string(),
                    value_date: Some(NaiveDate::parse_from_str(&val_date_str, "%d/%m/%Y").unwrap().format("%Y-%m-%d").to_string()),
                    description,
                    reference_number: None,
                    tx_type,
                    amount,
                    balance,
                };
                transactions.push(tx);
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
