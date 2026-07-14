use crate::layout::Line;
use xfina_models::{
    MutualFundsAccount, MfProfile, MfHolders, MfHolder, MfSummary, MfInvestment, MfHoldings,
    MfHolding, MfTransactions, MfTransaction, MfTransactionType, XfinaMutualFundsAccount,
    XfinaMutualFundsHolding, XfinaMutualFundsTransaction, parse_indian_date
};
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn parse_cas_lines(pages_lines: Vec<Vec<Line>>) -> Result<MutualFundsAccount, String> {
    let mut account = MutualFundsAccount {
        r#type: "mutualfunds".to_string(),
        masked_acc_number: String::new(),
        version: "1.1".to_string(),
        linked_acc_ref: String::new(),
        profile: None,
        summary: None,
        transactions: None,
        xfina: Some(XfinaMutualFundsAccount::default()),
    };

    let mut holder = MfHolder::default();
    let mut holdings = Vec::new();
    let mut all_transactions: Vec<MfTransaction> = Vec::new();

    let mut current_holding: Option<MfHolding> = None;
    let mut current_folio_number: Option<String> = None;
    let mut in_investor_info = false;
    
    let mut statement_start_date = None;
    let mut statement_end_date = None;

    let date_re = Regex::new(r"^\d{2}-\S{3}-\d{4}").unwrap();

    for lines in pages_lines {
        for line in lines {
            let text = line.text.trim();
            if text.is_empty() { continue; }
            let lower_text = text.to_lowercase();

            // Statement dates
            if statement_start_date.is_none() && lower_text.contains(" to ") {
                let parts: Vec<&str> = lower_text.split(" to ").collect();
                if parts.len() == 2 && date_re.is_match(parts[0].trim()) && date_re.is_match(parts[1].trim()) {
                    let original_parts: Vec<&str> = text.splitn(2, |c| c == 'T' || c == 't').collect();
                    if original_parts.len() == 2 {
                        let p1 = original_parts[0].trim().to_string();
                        let p2 = original_parts[1][1..].trim().to_string();
                        if date_re.is_match(&p1) {
                            statement_start_date = Some(parse_indian_date(&p1));
                            statement_end_date = Some(parse_indian_date(&p2));
                        }
                    }
                }
            }

            // Extract investor info
            if lower_text.starts_with("folio no:") {
                let parts: Vec<&str> = text.split("PAN:").collect();
                if !parts.is_empty() {
                    let folio_str = parts[0].replace("Folio No:", "").trim().to_string();
                    current_folio_number = Some(folio_str);
                }
                
                if parts.len() > 1 && holder.pan.is_none() {
                    let pan_parts: Vec<&str> = parts[1].split_whitespace().collect();
                    if !pan_parts.is_empty() {
                        holder.pan = Some(pan_parts[0].to_string());
                    }
                }
                in_investor_info = true;
                continue;
            } else if in_investor_info {
                if holder.name.is_empty() && !lower_text.contains("isin:") {
                    holder.name = text.to_string();
                }
                in_investor_info = false;
            }

            if let Some(idx) = text.find("Email ID:") {
                let parts: Vec<&str> = text[idx..].split_whitespace().collect();
                if parts.len() > 2 && holder.email.is_none() {
                    holder.email = Some(parts[2].to_string());
                }
            }

            // Detect new scheme
            if lower_text.contains("isin:") {
                if let Some(h) = current_holding.take() {
                    holdings.push(h);
                }
                
                let mut isin = None;
                if let Some(idx) = text.find("ISIN:") {
                    let parts: Vec<&str> = text[idx..].split_whitespace().collect();
                    if parts.len() > 1 {
                        let isin_raw = parts[1];
                        let isin_clean = isin_raw.split('(').next().unwrap_or(isin_raw).trim();
                        isin = Some(isin_clean.to_string());
                    }
                }
                
                let mut name_clean = text.to_string();
                if let Some(idx) = name_clean.find(" - ISIN:") {
                    name_clean = name_clean[..idx].to_string();
                }
                if let Some(idx) = name_clean.find('-') {
                    if !name_clean[..idx].contains(' ') {
                        name_clean = name_clean[idx+1..].trim().to_string();
                    }
                }
                
                let mut xfina = XfinaMutualFundsHolding::default();
                xfina.scheme_name = Some(name_clean);

                current_holding = Some(MfHolding {
                    folio_no: current_folio_number.clone(),
                    isin,
                    xfina: Some(xfina),
                    ..Default::default()
                });
                continue;
            }

            if lower_text.starts_with("closing unit balance:") {
                if let Some(h) = current_holding.as_mut() {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    for (i, p) in parts.iter().enumerate() {
                        let parse_dec = |s: &str| -> Decimal {
                            Decimal::from_str(&s.replace(",", "")).unwrap_or_default()
                        };

                        if p.to_lowercase() == "balance:" && i + 1 < parts.len() {
                            h.units = parse_dec(parts[i+1]);
                            h.closing_units = h.units; // Schema duplicate
                        }
                        if p.to_lowercase() == "value:" && i > 0 && parts[i-1].to_lowercase() == "cost" && i + 1 < parts.len() {
                            if let Some(x) = h.xfina.as_mut() {
                                x.total_invested = parse_dec(parts[i+1]);
                            }
                        }
                        if p.to_lowercase() == "nav" && i + 2 < parts.len() && parts[i+1].to_lowercase() == "on" {
                            if let Some(x) = h.xfina.as_mut() {
                                let date_str = parse_indian_date(&parts[i+2].replace(":", ""));
                                x.nav_date = date_str.parse().ok();
                            }
                        }
                        
                        if p.to_lowercase() == "inr" && i + 1 < parts.len() {
                            let val = parse_dec(parts[i+1]);
                            
                            let mut is_nav = false;
                            let mut is_market = false;
                            for j in 1..=4 {
                                if i >= j {
                                    if parts[i-j].to_lowercase() == "nav" {
                                        is_nav = true;
                                        break;
                                    }
                                    if parts[i-j].to_lowercase() == "market" {
                                        is_market = true;
                                        break;
                                    }
                                }
                            }
                            
                            if is_nav {
                                h.nav = val;
                                h.rate = val; // We also set rate to nav
                            } else if is_market {
                                if let Some(x) = h.xfina.as_mut() {
                                    x.current_value = val;
                                }
                            }
                        }
                    }
                }
                continue;
            }

            if let Some(h) = current_holding.as_mut() {
                if date_re.is_match(text) {
                    if text.contains("*** Stamp Duty ***") || text.contains("*** STT ***") {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        if let Some(last_val) = parts.last() {
                            let fee = Decimal::from_str(&last_val.replace(",", "")).unwrap_or_default().abs();
                            if let Some(txn) = all_transactions.last_mut() {
                                if let Some(x) = &mut txn.xfina {
                                    x.fees = fee;
                                }
                            }
                        }
                        continue;
                    }

                    let parts: Vec<&str> = text.split_whitespace().collect();
                    if parts.len() < 3 || text.to_lowercase().contains(" to ") {
                        continue;
                    }
                    
                    let date = parts.first().unwrap_or(&"").to_string();
                    let len = parts.len();
                    
                    let mut amount = Decimal::default();
                    let mut units = Decimal::default();
                    let mut nav = Decimal::default();
                    let mut raw_desc = String::new();
                    
                    if len >= 5 {
                        let parse_dec = |s: &str| -> Decimal {
                            Decimal::from_str(&s.replace(",", "").replace("(", "-").replace(")", "")).unwrap_or_default()
                        };
                        
                        nav = parse_dec(parts[len - 2]);
                        units = parse_dec(parts[len - 3]);
                        amount = parse_dec(parts[len - 4]);
                        
                        raw_desc = parts[1..(len - 4)].join(" ");
                    } else {
                        raw_desc = parts[1..].join(" ");
                    }
                    
                    let desc_lower = raw_desc.to_lowercase();
                    let tx_type = if desc_lower.contains("redemption") || desc_lower.contains("switch out") || desc_lower.contains("sell") {
                        MfTransactionType::Sell
                    } else {
                        MfTransactionType::Buy
                    };

                    let date_parsed = parse_indian_date(&date);
                    
                    let mut xfina = XfinaMutualFundsTransaction::default();
                    xfina.units = units;

                    let mut txn = MfTransaction {
                        isin: h.isin.clone(),
                        amount,
                        nav,
                        r#type: Some(tx_type),
                        narration: Some(raw_desc),
                        order_date: date_parsed.parse().ok(),
                        execution_date: date_parsed.parse().ok(),
                        xfina: Some(xfina),
                        ..Default::default()
                    };
                    
                    all_transactions.push(txn);
                }
            }
        }
    }

    if let Some(h) = current_holding {
        holdings.push(h);
    }

    if !holder.name.is_empty() {
        account.profile = Some(MfProfile {
            holders: MfHolders {
                r#type: None,
                holder: vec![holder],
            }
        });
    }

    if !holdings.is_empty() {
        account.summary = Some(MfSummary {
            investment_value: Decimal::default(),
            current_value: Decimal::default(),
            investment: MfInvestment {
                holdings: MfHoldings {
                    holding: holdings,
                }
            }
        });
    }

    if !all_transactions.is_empty() {
        account.transactions = Some(MfTransactions {
            start_date: statement_start_date.and_then(|d| d.parse().ok()),
            end_date: statement_end_date.and_then(|d| d.parse().ok()),
            transaction: all_transactions,
        });
    }

    Ok(account)
}
