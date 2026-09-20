#![cfg(feature = "rt-sbi-forex-card")]

//! The SBI rate parser against the whole published archive.
//!
//! Every sheet SBI has published since 2020 is kept in a sibling repository,
//! alongside the USD series a previous Python implementation derived from it.
//! This reads every sheet and diffs the dollar's TT rates against that series.
//!
//! Skipped when the archive is not checked out, like the corpus tests. Set
//! `XFINA_SBI_FX_ARCHIVE` to read it from somewhere other than the default.
//!
//! Two differences from the old series are expected rather than tolerated:
//!
//! - rows it published as `0.0`, which is how the sheet writes "not quoted"
//!   and not a price anything traded at. Those are absent here.
//! - days it filed under the date the sheet was fetched rather than the date
//!   the sheet prints. A sheet re-downloaded on a day SBI did not publish is
//!   the previous day's sheet, and belongs under the day it quotes.
//!
//! Anything else is a defect, and the test names it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use xfina::models::request::ParseRequest;
use xfina::reference_rates::sbi_forex_card::parse_sbi_forex_card_rates;

fn archive() -> Option<PathBuf> {
    let root = std::env::var("XFINA_SBI_FX_ARCHIVE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../financial-data/src/sbi-fx-card-rates"));
    root.is_dir().then_some(root)
}

/// The USD series the previous implementation published, by date.
fn published(root: &Path) -> BTreeMap<String, (f64, f64)> {
    let derived = root
        .parent()
        .and_then(Path::parent)
        .map(|r| r.join("docs/sbi-fx-card-rates"))
        .unwrap_or_default();

    let mut out = BTreeMap::new();
    let Ok(years) = fs::read_dir(&derived) else {
        return out;
    };
    for year in years.flatten() {
        let Ok(raw) = fs::read_to_string(year.path().join("USD.json")) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        for row in value["data"].as_array().into_iter().flatten() {
            let (Some(date), Some(buy), Some(sell)) =
                (row[0].as_str(), row[1].as_f64(), row[2].as_f64())
            else {
                continue;
            };
            out.insert(date.to_string(), (buy, sell));
        }
    }
    out
}

fn sheets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for year in fs::read_dir(root).into_iter().flatten().flatten() {
        for sheet in fs::read_dir(year.path()).into_iter().flatten().flatten() {
            let path = sheet.path();
            if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn reads_the_whole_published_archive() {
    let Some(root) = archive() else {
        println!("SBI rate archive not present; skipping");
        return;
    };
    let published = published(&root);
    let sheets = sheets(&root);
    assert!(!sheets.is_empty(), "archive present but holds no sheets");

    let mut parsed: BTreeMap<String, (Option<f64>, Option<f64>)> = BTreeMap::new();
    let mut refused: Vec<String> = Vec::new();
    let mut disagreed: Vec<String> = Vec::new();

    for path in &sheets {
        // The sheet's own name only ever labels a failure; the date under
        // which a sheet's rates are filed always comes from inside it.
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        let bytes = fs::read(path).unwrap();
        match parse_sbi_forex_card_rates(ParseRequest::new(&bytes)) {
            Ok(result) => {
                let sheet = result.data;
                let usd = sheet.currency("USD");
                let row = (
                    usd.and_then(|c| c.get("tt_buy")).and_then(decimal),
                    usd.and_then(|c| c.get("tt_sell")).and_then(decimal),
                );
                let date = sheet.date.to_string();
                // Several files are the same sheet archived twice. They must
                // read the same, or one of them was read wrongly.
                if let Some(seen) = parsed.get(&date) {
                    if *seen != row {
                        disagreed.push(format!(
                            "  {date}: two sheets for the same day read differently"
                        ));
                    }
                }
                parsed.insert(date, row);
            }
            Err(e) => refused.push(format!("  {name}: {}", e.kind())),
        }
    }

    let mut wrong: Vec<String> = Vec::new();
    let (mut matched, mut was_zero, mut now_absent) = (0usize, 0usize, 0usize);

    for (date, (want_buy, want_sell)) in &published {
        let Some((got_buy, got_sell)) = parsed.get(date) else {
            // The old series filed this under a fetch date; the sheet itself
            // says it belongs to an earlier day, which is already covered.
            now_absent += 1;
            continue;
        };
        if *want_buy == 0.0 || *want_sell == 0.0 {
            was_zero += 1;
            // A zero was never a price. It must not have come back as one.
            if got_buy.map(|v| v == 0.0).unwrap_or(false)
                || got_sell.map(|v| v == 0.0).unwrap_or(false)
            {
                wrong.push(format!("  {date}: a zero was published as a rate"));
            }
            continue;
        }
        match (got_buy, got_sell) {
            (Some(buy), Some(sell)) if close(*buy, *want_buy) && close(*sell, *want_sell) => {
                matched += 1
            }
            (buy, sell) => wrong.push(format!(
                "  {date}: read {buy:?}/{sell:?}, published {want_buy}/{want_sell}"
            )),
        }
    }

    let gained = parsed
        .keys()
        .filter(|d| !published.contains_key(*d))
        .count();
    println!(
        "{} sheets read, {} refused; USD TT matched on {matched} days, \
         {was_zero} previously-zero rows dropped, {now_absent} re-dated, \
         {gained} days gained",
        sheets.len() - refused.len(),
        refused.len(),
    );
    if !refused.is_empty() {
        println!("refused:\n{}", refused.join("\n"));
    }

    assert!(
        disagreed.is_empty(),
        "the same day read two different ways:\n{}",
        disagreed.join("\n")
    );
    assert!(
        wrong.is_empty(),
        "{} days disagree with the published series:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(
        matched > 1000,
        "only {matched} days matched; expected most of the archive"
    );
}

/// The published series carries rates as JSON floats, so the comparison is a
/// float one. A tenth of a paisa is far below anything the sheet prints.
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.001
}

fn decimal(value: rust_decimal::Decimal) -> Option<f64> {
    use rust_decimal::prelude::ToPrimitive;
    value.to_f64()
}
