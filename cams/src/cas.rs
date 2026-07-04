use crate::layout::Line;
use financial_extract_models::{Portfolio, InvestorInfo, Asset, Transaction};
use regex::Regex;

pub fn parse_cas_lines(pages_lines: Vec<Vec<Line>>) -> Result<Portfolio, String> {
    let mut portfolio = Portfolio {
        investor_info: InvestorInfo::default(),
        statement_start_date: None,
        statement_end_date: None,
        generated_date: None,
        assets: Vec::new(),
    };

    let mut current_asset: Option<Asset> = None;
    let mut current_folio_number: Option<String> = None;
    let mut in_investor_info = false;
    
    let date_re = Regex::new(r"^\d{2}-\S{3}-\d{4}").unwrap();

    for lines in pages_lines {
        for line in lines {
            let text = line.text.trim();
            if text.is_empty() { continue; }
            let lower_text = text.to_lowercase();

            // Statement dates (e.g. 01-Apr-2026 To 20-Jun-2026)
            if portfolio.statement_start_date.is_none() && lower_text.contains(" to ") {
                let parts: Vec<&str> = lower_text.split(" to ").collect();
                if parts.len() == 2 && date_re.is_match(parts[0].trim()) && date_re.is_match(parts[1].trim()) {
                    let original_parts: Vec<&str> = text.splitn(2, |c| c == 'T' || c == 't').collect();
                    if original_parts.len() == 2 {
                        let p1 = original_parts[0].trim().to_string();
                        let p2 = original_parts[1][1..].trim().to_string(); // Skip the 'o ' in 'To '
                        if date_re.is_match(&p1) {
                            portfolio.statement_start_date = Some(financial_extract_models::parse_indian_date(&p1));
                            portfolio.statement_end_date = Some(financial_extract_models::parse_indian_date(&p2));
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
                
                if parts.len() > 1 && portfolio.investor_info.pan.is_none() {
                    let pan_parts: Vec<&str> = parts[1].split_whitespace().collect();
                    if !pan_parts.is_empty() {
                        portfolio.investor_info.pan = Some(pan_parts[0].to_string());
                    }
                }
                in_investor_info = true;
                continue;
            } else if in_investor_info {
                if portfolio.investor_info.name.is_none() && !lower_text.contains("isin:") {
                    portfolio.investor_info.name = Some(text.to_string());
                }
                in_investor_info = false;
            }

            if let Some(idx) = text.find("Email ID:") {
                let parts: Vec<&str> = text[idx..].split_whitespace().collect();
                if parts.len() > 2 && portfolio.investor_info.email.is_none() {
                    portfolio.investor_info.email = Some(parts[2].to_string());
                }
            }

            // Detect new scheme
            if lower_text.contains("isin:") {
                if let Some(asset) = current_asset.take() {
                    portfolio.assets.push(asset);
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
                
                current_asset = Some(Asset {
                    name: name_clean, 
                    folio_number: current_folio_number.clone(),
                    isin,
                    symbol: None,
                    category: None,
                    period_units: 0.0,
                    period_invested_value: 0.0,
                    period_realized_value: 0.0,
                    total_units: 0.0,
                    total_cost_basis: 0.0,
                    current_nav: None,
                    current_nav_date: None,
                    current_value: None,
                    transactions: Vec::new(),
                });
                continue;
            }

            // Summary values parsing
            if lower_text.contains("opening unit balance:") {
                if let Some(_asset) = current_asset.as_mut() {
                    // Try to parse opening balance just in case
                }
            }
            if lower_text.starts_with("closing unit balance:") {
                if let Some(asset) = current_asset.as_mut() {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    for (i, p) in parts.iter().enumerate() {
                        if p.to_lowercase() == "balance:" && i + 1 < parts.len() {
                            asset.total_units = parts[i+1].replace(",", "").parse().unwrap_or(0.0);
                        }
                        if p.to_lowercase() == "value:" && i > 0 && parts[i-1].to_lowercase() == "cost" && i + 1 < parts.len() {
                            asset.total_cost_basis = parts[i+1].replace(",", "").parse().unwrap_or(0.0);
                        }
                        if p.to_lowercase() == "nav" && i + 2 < parts.len() && parts[i+1].to_lowercase() == "on" {
                            asset.current_nav_date = Some(financial_extract_models::parse_indian_date(&parts[i+2].replace(":", "")));
                        }
                        
                        // Parse "INR <value>"
                        if p.to_lowercase() == "inr" && i + 1 < parts.len() {
                            let val = parts[i+1].replace(",", "").parse().unwrap_or(0.0);
                            
                            // It's NAV if "NAV" appears recently before INR
                            // Check previous tokens up to 4 words back
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
                                asset.current_nav = Some(val);
                            } else if is_market {
                                asset.current_value = Some(val);
                            }
                        }
                    }
                }
                continue;
            }

            if let Some(asset) = current_asset.as_mut() {
                if date_re.is_match(text) {
                    if text.contains("*** Stamp Duty ***") || text.contains("*** STT ***") {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        if let Some(last_val) = parts.last() {
                            let fee: f64 = last_val.replace(",", "").parse().unwrap_or(0.0);
                            if let Some(txn) = asset.transactions.last_mut() {
                                txn.fee = Some(fee.abs());
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
                    
                    let mut amount = 0.0;
                    let mut units = 0.0;
                    let mut nav = None;
                    let mut balance = None;
                    let mut raw_desc = String::new();
                    
                    if len >= 5 {
                        let parse_num = |s: &str| -> f64 {
                            s.replace(",", "").replace("(", "-").replace(")", "").parse().unwrap_or(0.0)
                        };
                        
                        balance = Some(parse_num(parts[len - 1]));
                        nav = parts.get(len - 2).map(|s| parse_num(s));
                        units = parts.get(len - 3).map(|s| parse_num(s)).unwrap_or(0.0);
                        amount = parts.get(len - 4).map(|s| parse_num(s)).unwrap_or(0.0);
                        
                        raw_desc = parts[1..(len - 4)].join(" ");
                    } else {
                        raw_desc = parts[1..].join(" ");
                    }
                    
                    let desc_lower = raw_desc.to_lowercase();
                    let tx_type = if desc_lower.contains("redemption") || desc_lower.contains("switch out") || desc_lower.contains("sell") {
                        "SELL".to_string()
                    } else {
                        "BUY".to_string()
                    };
                    
                    asset.transactions.push(Transaction {
                        date: financial_extract_models::parse_indian_date(&date),
                        tx_type,
                        description: Some(raw_desc),
                        amount,
                        units,
                        nav,
                        balance,
                        fee: None,
                    });
                } else {
                    // It could be a continuation of the description for the last transaction
                    // For now, we skip wrapping text unless needed.
                }
            }
        }
    }

    if let Some(asset) = current_asset {
        portfolio.assets.push(asset);
    }

    Ok(portfolio)
}
