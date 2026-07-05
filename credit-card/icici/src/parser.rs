use calamine::{Reader, Xlsx, open_workbook_from_rs};
use std::io::Cursor;
use financial_extract_models::{CreditCardStatement, AccountSummary, CustomerInfo, CreditCardTransaction, RewardPointsSummary};

pub fn parse_icici_statement(bytes: &[u8]) -> Result<CreditCardStatement, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| format!("Failed to open workbook: {}", e))?;

    let sheet_names = workbook.sheet_names().to_owned();
    let first_sheet = sheet_names.first().ok_or("No sheets found in workbook")?;
    let range = workbook.worksheet_range(first_sheet)
        .map_err(|e| format!("Failed to get worksheet: {}", e))?;

    let mut stmt = CreditCardStatement::default();
    let mut in_transactions = false;

    let mut card_holder_name = String::new();
    let mut previous_balance = 0.0;
    let mut purchases = 0.0;
    let mut payments = 0.0;
    let mut total_due = 0.0;
    let mut min_due = 0.0;

    let mut default_rewards = 0;
    let mut owner_credit_breakdown = std::collections::HashMap::new();
    let mut owner_debit_breakdown = std::collections::HashMap::new();

    for row in range.rows() {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        if cells.is_empty() { continue; }

        let col0 = cells.get(0).map(|s| s.trim()).unwrap_or("");

        // Parse key-value headers
        if !in_transactions {
            match col0 {
                "Card Holder Name" => {
                    card_holder_name = cells.get(4).unwrap_or(&String::new()).trim().to_string();
                    stmt.customer_info = CustomerInfo {
                        name: card_holder_name.clone(),
                        ..Default::default()
                    };
                }
                "Previous Balance" => {
                    let val = cells.get(4).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    previous_balance = val.parse().unwrap_or(0.0);
                    stmt.statement_date = Some(financial_extract_models::parse_indian_date(cells.get(12).unwrap_or(&String::new()).trim()));
                }
                "Purchases and Other Charges" => {
                    let val = cells.get(4).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    purchases = val.parse().unwrap_or(0.0);
                    stmt.payment_due_date = Some(financial_extract_models::parse_indian_date(cells.get(12).unwrap_or(&String::new()).trim()));
                }
                "Payments and Other Credits" => {
                    let val = cells.get(4).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    payments = val.parse().unwrap_or(0.0);
                    let limit = cells.get(12).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    stmt.credit_limit = Some(limit.parse().unwrap_or(0.0));
                }
                "Total Amount Due" => {
                    let val = cells.get(4).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    total_due = val.parse().unwrap_or(0.0);
                    let limit = cells.get(12).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    stmt.available_limit = Some(limit.parse().unwrap_or(0.0));
                }
                "Minimum Amount Due" => {
                    let val = cells.get(4).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    min_due = val.parse().unwrap_or(0.0);
                    stmt.minimum_amount_due = Some(min_due);
                    
                    let limit = cells.get(12).unwrap_or(&String::new()).replace("INR", "").trim().to_string();
                    stmt.available_cash_limit = Some(limit.parse().unwrap_or(0.0));
                }
                "Cash Advances" => {
                    let period = cells.get(12).unwrap_or(&String::new()).trim().to_string();
                    if let Some((start, end)) = period.split_once(" TO ") {
                        stmt.statement_start_date = Some(financial_extract_models::parse_indian_date(start.trim()));
                        stmt.statement_end_date = Some(financial_extract_models::parse_indian_date(end.trim()));
                    }
                }
                "Transaction Date" => {
                    in_transactions = true;
                }
                _ => {}
            }
        } else {
            // Parse Transactions
            // 0=Date, 4=Details, 8=Amount, 12=Reward Points, 16=Ref Number
            let date = cells.get(0).map(|s| s.trim()).unwrap_or("");
            if date.is_empty() || date == "Transaction Date" { continue; } // skip empty or header
            
            let details = cells.get(4).map(|s| s.trim()).unwrap_or("").to_string();
            let amount_str = cells.get(8).map(|s| s.trim()).unwrap_or("").to_string();
            
            // Parse amount and type
            let is_credit = amount_str.ends_with("Cr.");
            let is_debit = amount_str.ends_with("Dr.");
            let tx_type = if is_credit { "Credit".to_string() } else if is_debit { "Debit".to_string() } else { "Unknown".to_string() };
            
            let amt_clean = amount_str.replace("Dr.", "").replace("Cr.", "").replace("INR", "").trim().to_string();
            let amount = amt_clean.parse::<f64>().unwrap_or(0.0).abs();
            
            let reward_points = cells.get(12).and_then(|s| s.trim().parse::<i32>().ok());
            
            let has_extra_cols = cells.iter().skip(17).any(|s| !s.trim().is_empty());
            let category = if has_extra_cols { Some("INTL".to_string()) } else { Some("IN".to_string()) };
            
            stmt.transactions.push(CreditCardTransaction {
                owner: card_holder_name.clone(), // default to card holder
                date: financial_extract_models::parse_indian_date(date),
                description: details,
                amount,
                tx_type: tx_type.clone(),
                reward_points,
                category,
            });
            
            // Aggregations
            if let Some(pts) = reward_points {
                default_rewards += pts;
            }
            if tx_type == "Credit" {
                *owner_credit_breakdown.entry(card_holder_name.clone()).or_insert(0.0) += amount;
            } else if tx_type == "Debit" {
                *owner_debit_breakdown.entry(card_holder_name.clone()).or_insert(0.0) += amount;
            }
        }
    }
    
    stmt.account_summary = Some(AccountSummary {
        opening_balance: previous_balance,
        payment_credit: payments,
        purchases_debits: purchases,
        finance_charges: 0.0, // not clearly separated in this ICICI extract
        total_dues: total_due,
        owner_credit_breakdown,
        owner_debit_breakdown,
    });
    
    stmt.reward_points_summary = Some(RewardPointsSummary {
        default_rewards,
        opening_balance: 0,
        earned: default_rewards,
        disbursed: 0,
        adjusted_lapsed: 0,
        closing_balance: 0,
        expiring_in_30_days: None,
        expiring_in_60_days: None,
    });
    
    Ok(stmt)
}
