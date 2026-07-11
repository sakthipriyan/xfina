use chrono::{NaiveDate, DateTime, Utc, TimeZone};
use rust_decimal::Decimal;
use xfina_models::credit_card::{
    CreditCardAccount, CcProfile, CcHolders, CcHolder, CcCards, CcCard, CardType, TypeChoice,
    CcSummary, PastDues, RewardPointsSummary, RewardProgram,
    CcTransactions, CcTransaction, XfinaCreditCardAccount, CcXfinaSummary, CcXfinaTransactions, CcXfinaTransaction,
};
use xfina_models::deposit::TransactionType;
use std::collections::HashMap;
use num_traits::cast::ToPrimitive;
use regex::Regex;

pub fn parse_hdfc_statement(content: &str, filename: Option<&str>) -> Result<CreditCardAccount, String> {
    let mut stmt = CreditCardAccount::default();
    stmt.r#type = "credit_card".to_string();
    stmt.version = 1.1;

    let mut address_parts = Vec::new();
    let mut holder = CcHolder::default();
    let mut xfina_account = XfinaCreditCardAccount::default();
    xfina_account.institution_name = Some("HDFC".to_string());
    
    if let Some(fname) = filename {
        let re = Regex::new(r"(\d{2}-\d{2}-\d{4})").unwrap();
        if let Some(caps) = re.captures(fname) {
            if let Some(m) = caps.get(1) {
                if let Ok(d) = NaiveDate::parse_from_str(m.as_str(), "%d-%m-%Y") {
                    let dt = d.and_hms_opt(0, 0, 0).unwrap();
                    let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                    xfina_account.generated_date = chrono::TimeZone::from_local_datetime(&ist_offset, &dt).single().map(|dt| dt.with_timezone(&Utc));
                    xfina_account.date_only = Some(true);
                }
            }
        }
    }
    let mut summary = CcSummary::default();
    let mut xfina_summary = CcXfinaSummary::default();
    
    let mut transactions_list = Vec::new();
    let mut xfina_txns = CcXfinaTransactions::default();
    
    let mut card_no = String::new();

    let lines: Vec<&str> = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let mut idx = 0;
    
    enum Section {
        Top,
        AccountCcSummary,
        PastDues,
        CcTransactions,
        RewardCcSummary,
        RewardProgram,
        None
    }
    let mut current_section = Section::Top;

    while idx < lines.len() {
        let line = lines[idx];
        if line == "Account Summary" {
            current_section = Section::AccountCcSummary;
            idx += 1;
            continue;
        } else if line.starts_with("Past Dues") {
            current_section = Section::PastDues;
            idx += 1;
            continue;
        } else if line == "Domestic / International Transactions" {
            current_section = Section::CcTransactions;
            idx += 1;
            continue;
        } else if line == "Reward Points Summary" {
            current_section = Section::RewardCcSummary;
            idx += 1;
            continue;
        } else if line == "Rewards Program Points Summary" {
            current_section = Section::RewardProgram;
            idx += 1;
            continue;
        } else if line.starts_with("State account branch GSTN") {
            current_section = Section::None;
        } else if line.starts_with("Card No:") {
            card_no = line.replace("Card No:", "").trim().to_string();
            stmt.masked_acc_number = card_no.clone();
            idx += 1;
            continue;
        } else if line.starts_with("AAN:") {
            xfina_account.aan = Some(line.replace("AAN:", "").trim().to_string());
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
                        "Name" => holder.name = val.to_string(),
                        "Address" => address_parts.push(val.to_string()),
                        "Payment Due Date" => summary.due_date = parse_date(val),
                        "Statement Date" => {
                            let d = parse_date(val);
                            summary.last_statement_date = d.clone();
                            if let Some(date) = d {
                                let dt = date.and_hms_opt(0, 0, 0).unwrap();
                                let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
                                xfina_account.generated_date = chrono::TimeZone::from_local_datetime(&ist_offset, &dt).single().map(|dt| dt.with_timezone(&Utc));
                                xfina_account.date_only = Some(true);
                            }
                        }
                        "Total Amount Due" => summary.total_due_amount = parse_decimal(val),
                        "Minimum Amount Due" => summary.min_due_amount = parse_decimal(val),
                        "Credit Limit" => summary.credit_limit = parse_decimal(val),
                        "Available Limit" => summary.available_credit = parse_decimal(val),
                        "Available Cash limit" => summary.cash_limit = parse_decimal(val),
                        _ => {}
                    }
                }
            }
            Section::AccountCcSummary => {
                if parts[0] == "Opening Bal" && idx + 1 < lines.len() {
                    let val_parts: Vec<&str> = lines[idx + 1].split("~|~").map(|p| p.trim()).collect();
                    if val_parts.len() >= 9 {
                        xfina_summary.opening_balance = parse_decimal(val_parts[0]);
                        xfina_summary.payment_credit = parse_decimal(val_parts[2]);
                        xfina_summary.purchases_debits = parse_decimal(val_parts[4]);
                        summary.finance_charges = parse_decimal(val_parts[6]);
                    }
                    idx += 1;
                }
            }
            Section::PastDues => {
                if parts[0] == "Overlimit" && idx + 1 < lines.len() {
                    let val_parts: Vec<&str> = lines[idx + 1].split("~|~").map(|p| p.trim()).collect();
                    if val_parts.len() >= 6 {
                        xfina_summary.past_dues = Some(PastDues {
                            overlimit: parse_decimal(val_parts[0]).unwrap_or_default(),
                            three_months: parse_decimal(val_parts[1]).unwrap_or_default(),
                            two_months: parse_decimal(val_parts[2]).unwrap_or_default(),
                            one_month: parse_decimal(val_parts[3]).unwrap_or_default(),
                        });
                        summary.current_due = parse_decimal(val_parts[4]);
                    }
                    idx += 1;
                }
            }
            Section::CcTransactions => {
                if parts.len() >= 5 && parts[0] != "Transaction type" {
                    let owner = parts.get(1).unwrap_or(&"").trim().to_string();
                    let date = parse_date(parts.get(2).unwrap_or(&""));
                    let desc = parts.get(3).unwrap_or(&"").to_string();
                    let amount = parse_decimal(parts.get(4).unwrap_or(&"")).unwrap_or_default().abs();
                    let ty = parts.get(5).unwrap_or(&"");
                    let txn_type = if *ty == "Cr" { TransactionType::Credit } else { TransactionType::Debit };
                    let rp_str = parts.get(6).unwrap_or(&"").replace("+", "");
                    let reward_points = rp_str.trim().parse::<i32>().ok();
                    
                    let mut tx_xfina = CcXfinaTransaction::default();
                    tx_xfina.owner = Some(owner);
                    tx_xfina.reward_points = reward_points;

                    transactions_list.push(CcTransaction {
                        txn_date: date.clone(),
                        value_date: date,
                        narration: desc,
                        amount,
                        txn_type,
                        txn_id: None,
                        statement_date: None,
                        mcc: None,
                        masked_card_number: None,
                        xfina: Some(tx_xfina),
                    });
                }
            }
            Section::RewardCcSummary => {
                if parts[0] == "Opening Balance" && idx + 1 < lines.len() {
                    let val_parts: Vec<&str> = lines[idx + 1].split("~|~").map(|p| p.trim()).collect();
                    if val_parts.len() >= 5 {
                        xfina_summary.reward_points_summary = Some(RewardPointsSummary {
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
                    xfina_summary.reward_programs.push(RewardProgram {
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
        holder.address = Some(address_parts.join(", "));
    }
    
    if !card_no.is_empty() {
        holder.cards = Some(CcCards {
            card: vec![CcCard {
                card_type: CardType::Others,
                primary: TypeChoice::Yes,
                masked_card_number: card_no,
                issued_date: None,
            }],
        });
    }

    stmt.profile = Some(CcProfile {
        holders: CcHolders {
            holder: vec![holder],
        }
    });
    
    // Compute aggregations
    let mut default_rewards = 0;
    let mut owner_credit_breakdown = HashMap::new();
    let mut owner_debit_breakdown = HashMap::new();

    for txn in &transactions_list {
        if let Some(ref xfina) = txn.xfina {
            if let Some(pts) = xfina.reward_points {
                default_rewards += pts;
            }
            let owner = if let Some(o) = &xfina.owner {
                if o.is_empty() { "Unknown".to_string() } else { o.clone() }
            } else {
                "Unknown".to_string()
            };
            
            use rust_decimal::prelude::ToPrimitive;
            let amt = txn.amount.to_f64().unwrap_or(0.0);
            
            if txn.txn_type == TransactionType::Credit {
                *owner_credit_breakdown.entry(owner).or_insert(0.0) += amt;
            } else {
                *owner_debit_breakdown.entry(owner).or_insert(0.0) += amt;
            }
        }
    }

    if let Some(ref mut rs) = xfina_summary.reward_points_summary {
        rs.default_rewards = default_rewards;
    }
    xfina_summary.owner_credit_breakdown = owner_credit_breakdown;
    xfina_summary.owner_debit_breakdown = owner_debit_breakdown;
    
    summary.xfina = Some(xfina_summary);
    stmt.summary = Some(summary);

    transactions_list.sort_by(|a, b| a.txn_date.cmp(&b.txn_date));

    let mut txns = CcTransactions::default();
    if let Some(first) = transactions_list.first() {
        txns.start_date = first.txn_date.clone();
        xfina_txns.start_date_derived = Some(true);
    }
    if let Some(last) = transactions_list.last() {
        txns.end_date = last.txn_date.clone();
        xfina_txns.end_date_derived = Some(true);
    }
    txns.transaction = transactions_list;
    txns.xfina = Some(xfina_txns);
    
    stmt.transactions = Some(txns);
    stmt.xfina = Some(xfina_account);
    
    Ok(stmt)
}

fn parse_decimal(val: &str) -> Option<Decimal> {
    let clean = val.replace(",", "");
    clean.parse::<Decimal>().ok()
}

fn parse_i32(val: &str) -> Option<i32> {
    let clean = val.replace(",", "");
    clean.parse::<i32>().ok()
}

fn parse_date(val: &str) -> Option<NaiveDate> {
    let iso = xfina_models::parse_indian_date(val);
    let s = iso.split('T').next().unwrap_or(&iso);
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}
