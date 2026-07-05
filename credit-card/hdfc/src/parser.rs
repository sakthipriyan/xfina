use financial_extract_models::credit_card::{
    AccountSummary, CreditCardStatement, CreditCardTransaction, CustomerInfo, PastDues,
    RewardPointsSummary, RewardProgram,
};

pub fn parse_hdfc_statement(content: &str) -> Result<CreditCardStatement, String> {
    let mut stmt = CreditCardStatement::default();
    let mut address_parts = Vec::new();

    let lines: Vec<&str> = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let mut idx = 0;
    
    enum Section {
        Top,
        AccountSummary,
        PastDues,
        Transactions,
        RewardSummary,
        RewardProgram,
        None
    }
    let mut current_section = Section::Top;

    while idx < lines.len() {
        let line = lines[idx];
        if line == "Account Summary" {
            current_section = Section::AccountSummary;
            idx += 1;
            continue;
        } else if line.starts_with("Past Dues") {
            current_section = Section::PastDues;
            idx += 1;
            continue;
        } else if line == "Domestic / International Transactions" {
            current_section = Section::Transactions;
            idx += 1;
            continue;
        } else if line == "Reward Points Summary" {
            current_section = Section::RewardSummary;
            idx += 1;
            continue;
        } else if line == "Rewards Program Points Summary" {
            current_section = Section::RewardProgram;
            idx += 1;
            continue;
        } else if line.starts_with("State account branch GSTN") {
            current_section = Section::None;
        } else if line.starts_with("Card No:") {
            stmt.card_no = Some(line.replace("Card No:", "").trim().to_string());
            idx += 1;
            continue;
        } else if line.starts_with("AAN:") {
            stmt.aan = Some(line.replace("AAN:", "").trim().to_string());
            idx += 1;
            continue;
        }

        let parts: Vec<&str> = line.split("~|~").map(|p| p.trim()).collect();
        match current_section {
            Section::Top => {
                if parts.len() >= 2 {
                    let key = parts[0];
                    let val = parts[1];
                    match key {
                        "Name" => stmt.customer_info.name = val.to_string(),
                        "Address" => address_parts.push(val.to_string()),
                        "Customer GSTN" => stmt.customer_info.customer_gstn = if val.is_empty() { None } else { Some(val.to_string()) },
                        "Payment Due Date" => stmt.payment_due_date = Some(financial_extract_models::parse_indian_date(val)),
                        "Statement Date" => stmt.statement_date = Some(financial_extract_models::parse_indian_date(val)),
                        "Total Amount Due" => stmt.total_amount_due = parse_f64(val),
                        "Minimum Amount Due" => stmt.minimum_amount_due = parse_f64(val),
                        "Credit Limit" => stmt.credit_limit = parse_f64(val),
                        "Available Limit" => stmt.available_limit = parse_f64(val),
                        "Available Cash limit" => stmt.available_cash_limit = parse_f64(val),
                        _ => {}
                    }
                }
            }
            Section::AccountSummary => {
                // Header is at idx, values at idx + 1
                if parts[0] == "Opening Bal" && idx + 1 < lines.len() {
                    let val_parts: Vec<&str> = lines[idx + 1].split("~|~").map(|p| p.trim()).collect();
                    if val_parts.len() >= 9 {
                        stmt.account_summary = Some(AccountSummary {
                            opening_balance: parse_f64(val_parts[0]).unwrap_or(0.0),
                            payment_credit: parse_f64(val_parts[2]).unwrap_or(0.0),
                            purchases_debits: parse_f64(val_parts[4]).unwrap_or(0.0),
                            finance_charges: parse_f64(val_parts[6]).unwrap_or(0.0),
                            total_dues: parse_f64(val_parts[8]).unwrap_or(0.0),
                            owner_credit_breakdown: std::collections::HashMap::new(),
                            owner_debit_breakdown: std::collections::HashMap::new(),
                        });
                    }
                    idx += 1; // skip next line since we processed it
                }
            }
            Section::PastDues => {
                if parts[0] == "Overlimit" && idx + 1 < lines.len() {
                    let val_parts: Vec<&str> = lines[idx + 1].split("~|~").map(|p| p.trim()).collect();
                    if val_parts.len() >= 6 {
                        stmt.past_dues = Some(PastDues {
                            overlimit: parse_f64(val_parts[0]).unwrap_or(0.0),
                            three_months: parse_f64(val_parts[1]).unwrap_or(0.0),
                            two_months: parse_f64(val_parts[2]).unwrap_or(0.0),
                            one_month: parse_f64(val_parts[3]).unwrap_or(0.0),
                            current_dues: parse_f64(val_parts[4]).unwrap_or(0.0),
                            minimum_amount_due: parse_f64(val_parts[5]).unwrap_or(0.0),
                        });
                    }
                    idx += 1; // skip next line since we processed it
                }
            }
            Section::Transactions => {
                if parts.len() >= 5 && parts[0] != "Transaction type" {
                    // Domestic~|~SAKTHI PRIYAN H ~|~30/05/2026 09:41:11~|~DESC~|~464.00~|~~|~+ 15
                    let cat = parts[0].trim();
                    let category = if cat == "International" {
                        Some("INTL".to_string())
                    } else if cat == "Domestic" {
                        Some("IN".to_string())
                    } else {
                        None
                    };
                    let owner = parts.get(1).unwrap_or(&"").trim().to_string();
                    let date = financial_extract_models::parse_indian_date(parts.get(2).unwrap_or(&""));
                    let desc = parts.get(3).unwrap_or(&"").to_string();
                    let amount = parse_f64(parts.get(4).unwrap_or(&"")).unwrap_or(0.0).abs();
                    let ty = parts.get(5).unwrap_or(&"");
                    let tx_type = if *ty == "Cr" { "Credit".to_string() } else { "Debit".to_string() };
                    let rp_str = parts.get(6).unwrap_or(&"").replace("+", "");
                    let reward_points = rp_str.trim().parse::<i32>().ok();
                    
                    stmt.transactions.push(CreditCardTransaction {
                        owner,
                        date,
                        description: desc,
                        amount,
                        tx_type,
                        reward_points,
                        category,
                    });
                }
            }
            Section::RewardSummary => {
                if parts[0] == "Opening Balance" && idx + 1 < lines.len() {
                    let val_parts: Vec<&str> = lines[idx + 1].split("~|~").map(|p| p.trim()).collect();
                    if val_parts.len() >= 5 {
                        stmt.reward_points_summary = Some(RewardPointsSummary {
                            opening_balance: parse_i32(val_parts[0]).unwrap_or(0),
                            earned: parse_i32(val_parts[1]).unwrap_or(0),
                            disbursed: parse_i32(val_parts[2]).unwrap_or(0),
                            adjusted_lapsed: parse_i32(val_parts[3]).unwrap_or(0),
                            closing_balance: parse_i32(val_parts[4]).unwrap_or(0),
                            expiring_in_30_days: val_parts.get(5).and_then(|v| parse_i32(v)),
                            expiring_in_60_days: val_parts.get(6).and_then(|v| parse_i32(v)),
                            default_rewards: 0,
                        });
                    }
                    idx += 1;
                }
            }
            Section::RewardProgram => {
                if parts.len() >= 2 && parts[0] != "Programs" {
                    stmt.reward_programs.push(RewardProgram {
                        program: parts[0].to_string(),
                        bonus_points: parse_i32(parts[1]).unwrap_or(0),
                    });
                }
            }
            Section::None => {}
        }
        idx += 1;
    }
    
    if !address_parts.is_empty() {
        stmt.customer_info.address = address_parts.join(", ");
    }
    
    // Compute aggregations
    let mut default_rewards = 0;
    let mut owner_credit_breakdown = std::collections::HashMap::new();
    let mut owner_debit_breakdown = std::collections::HashMap::new();

    for txn in &stmt.transactions {
        if let Some(pts) = txn.reward_points {
            default_rewards += pts;
        }
        let owner = if txn.owner.is_empty() { "Unknown".to_string() } else { txn.owner.clone() };
        if txn.tx_type == "Credit" {
            *owner_credit_breakdown.entry(owner).or_insert(0.0) += txn.amount;
        } else if txn.tx_type == "Debit" {
            *owner_debit_breakdown.entry(owner).or_insert(0.0) += txn.amount;
        }
    }

    if let Some(ref mut rs) = stmt.reward_points_summary {
        rs.default_rewards = default_rewards;
    }
    if let Some(ref mut as_sum) = stmt.account_summary {
        as_sum.owner_credit_breakdown = owner_credit_breakdown;
        as_sum.owner_debit_breakdown = owner_debit_breakdown;
    }
    
    Ok(stmt)
}

fn parse_f64(val: &str) -> Option<f64> {
    let clean = val.replace(",", "");
    clean.parse::<f64>().ok()
}

fn parse_i32(val: &str) -> Option<i32> {
    let clean = val.replace(",", "");
    clean.parse::<i32>().ok()
}
