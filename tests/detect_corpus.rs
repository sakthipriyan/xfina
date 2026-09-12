//! Detection against the real corpus, in both directions.
//!
//! The positive half asserts every statement resolves to the format that owns
//! its directory. The negative half is the one that matters more: the corpus
//! also holds files no parser supports -- emailed PDF variants of statements we
//! only read as spreadsheets, and a KFinTech CAS -- and those must come back
//! unrecognised. A parser that quietly claims one of them is the exact failure
//! detection exists to prevent.
//!
//! Runs only where the private corpus is checked out beside the repo; skipped
//! in CI like the other integration tests. Nothing from it is reproduced here:
//! failures are reported by directory and extension, never by filename.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use xfina::detect::Format;
use xfina::models::request::ParseRequest;

/// Corpus directory -> the format that owns it.
const OWNERS: &[(&str, Format)] = &[
    ("bank-accounts/hdfc", Format::BankHdfc),
    ("bank-accounts/icici", Format::BankIcici),
    ("bank-accounts/sbi", Format::BankSbi),
    ("bank-accounts/bob", Format::BankBob),
    ("bank-accounts/axis", Format::BankAxis),
    ("credit-cards/hdfc", Format::CardHdfc),
    ("credit-cards/icici", Format::CardIcici),
    ("credit-cards/axis", Format::CardAxis),
    ("mutual-funds/cams", Format::MutualFundsCams),
    ("intl-stocks/ibkr", Format::EquityIbkr),
];

fn corpus() -> Option<&'static Path> {
    let root = Path::new("../xfina-test-data");
    root.exists().then_some(root)
}

fn passwords(dir: &Path) -> BTreeMap<String, Vec<String>> {
    let Ok(raw) = fs::read_to_string(dir.join("passwords.json")) else {
        return BTreeMap::new();
    };
    let Ok(map) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    if let Some(obj) = map.as_object() {
        for (k, v) in obj {
            let list = match v {
                serde_json::Value::String(s) => vec![s.clone()],
                serde_json::Value::Array(a) => a
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect(),
                _ => continue,
            };
            out.insert(k.clone(), list);
        }
    }
    out
}

/// Detects with each candidate password, since an encrypted PDF cannot be
/// probed until it opens.
fn detect_with(bytes: &[u8], name: &str, pws: &[String]) -> Result<Format, String> {
    let attempt = |pw: Option<&str>| {
        xfina::detect(
            &ParseRequest::new(bytes)
                .with_filename(Some(name))
                .with_password(pw),
        )
        .map(|d| d.format)
    };
    match attempt(None) {
        Ok(f) => return Ok(f),
        Err(e) if !matches!(e.kind(), "password_required" | "incorrect_password") => {
            return Err(e.kind().to_string())
        }
        Err(_) => {}
    }
    for pw in pws {
        if let Ok(f) = attempt(Some(pw)) {
            return Ok(f);
        }
    }
    Err("password_required".to_string())
}

#[test]
fn every_statement_is_recognised_and_nothing_else_is() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping corpus test in CI");
        return;
    }
    let Some(root) = corpus() else {
        println!("../xfina-test-data not present; skipping");
        return;
    };

    let mut misdetected: Vec<String> = Vec::new();
    let mut claimed_unsupported: Vec<String> = Vec::new();
    let (mut ok, mut rejected) = (0usize, 0usize);

    for (dir, owner) in OWNERS {
        let raw = root.join(dir).join("raw");
        let Ok(entries) = fs::read_dir(&raw) else {
            continue;
        };
        let pw_map = passwords(&root.join(dir));
        let supported_ext = supported_extension(*owner);

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if name.starts_with('.') || ext == "json" {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else { continue };
            let pws = pw_map
                .get(name)
                .or_else(|| pw_map.get("default"))
                .cloned()
                .unwrap_or_default();

            let detected = detect_with(&bytes, name, &pws);
            let is_supported = supported_ext.contains(&ext.as_str());

            match (is_supported, detected) {
                (true, Ok(f)) if f == *owner => ok += 1,
                (true, Ok(f)) => {
                    misdetected.push(format!("  {dir}/*.{ext} detected as {f}, want {owner}"))
                }
                (true, Err(kind)) => {
                    misdetected.push(format!("  {dir}/*.{ext} not detected at all ({kind})"))
                }
                // An unsupported variant claimed by a parser is the failure
                // this test exists for.
                (false, Ok(f)) => {
                    claimed_unsupported.push(format!("  {dir}/*.{ext} claimed by {f}"))
                }
                (false, Err(_)) => rejected += 1,
            }
        }
    }

    println!("{ok} statements detected, {rejected} unsupported files correctly declined");
    assert!(
        misdetected.is_empty() && claimed_unsupported.is_empty(),
        "detection went wrong:\n{}\n{}",
        misdetected.join("\n"),
        claimed_unsupported.join("\n")
    );
    assert!(ok > 0, "corpus present but nothing was detected");
}

/// Extensions each format is actually able to read today. Anything else in a
/// format's raw directory is a variant we do not support yet.
fn supported_extension(f: Format) -> &'static [&'static str] {
    match f {
        Format::BankSbi | Format::MutualFundsCams => &["pdf"],
        Format::EquityIbkr => &["csv"],
        Format::CardAxis => &["xlsx"],
        _ => &["xls", "xlsx"],
    }
}

#[test]
fn a_renamed_statement_is_still_recognised() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        return;
    }
    let Some(root) = corpus() else { return };
    // Content is the authority: strip the institution's filename entirely and
    // detection must still land on the same format.
    let dir = root.join("bank-accounts/hdfc/raw");
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xls") {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        let detected = xfina::detect(&ParseRequest::new(&bytes).with_filename(Some("renamed.bin")))
            .expect("a renamed HDFC statement must still be detected");
        assert_eq!(
            detected.format,
            Format::BankHdfc,
            "a renamed statement must be identified from its content"
        );
        return;
    }
}

#[test]
fn an_encrypted_statement_asks_for_its_password() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        return;
    }
    let Some(root) = corpus() else { return };
    // Failing to open outranks failing to recognise: without this, every
    // parser declines an encrypted file and detection would report it as an
    // unknown format, sending a caller looking for a missing parser instead of
    // prompting for the password.
    let dir = root.join("mutual-funds/cams/raw");
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    let pw_map = passwords(&root.join("mutual-funds/cams"));
    let mut checked = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("pdf") {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let no_password = xfina::detect(&ParseRequest::new(&bytes).with_filename(Some(name)));
        let Err(e) = no_password else { continue };
        if e.kind() == "password_required" {
            let wrong = xfina::detect(
                &ParseRequest::new(&bytes)
                    .with_filename(Some(name))
                    .with_password(Some("not-the-password")),
            )
            .expect_err("a wrong password cannot open the file");
            assert_eq!(
                wrong.kind(),
                "incorrect_password",
                "a wrong password must be distinguishable from a missing one"
            );
            // And with the right one, it identifies itself.
            let pws = pw_map
                .get(name)
                .or_else(|| pw_map.get("default"))
                .cloned()
                .unwrap_or_default();
            for pw in &pws {
                if let Ok(d) = xfina::detect(
                    &ParseRequest::new(&bytes)
                        .with_filename(Some(name))
                        .with_password(Some(pw)),
                ) {
                    assert_eq!(d.format, Format::MutualFundsCams);
                    checked += 1;
                    break;
                }
            }
            break;
        }
    }
    println!("checked the password path on {checked} encrypted statement(s)");
}

#[test]
fn no_statement_is_hinted_at_the_wrong_institution() {
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        return;
    }
    let Some(root) = corpus() else { return };

    // A hint that misses is cheap: content decides, so detection just probes
    // one more candidate. A hint that fires *wrongly* is not -- for an
    // encrypted PDF, which cannot be probed at all, it is what names the
    // institution in the password prompt, and naming the wrong one sends
    // someone looking for a password they were never asked for.
    //
    // So misses are counted and reported; wrong answers fail. Some names carry
    // nothing to go on -- a second download saved as "(1).csv" -- and no
    // pattern can help those.
    let mut wrong = Vec::new();
    let (mut hinted, mut missed) = (0, 0);
    let mut encrypted_checked = 0;

    for (dir, owner) in OWNERS {
        let raw = root.join(dir).join("raw");
        let Ok(entries) = fs::read_dir(&raw) else {
            continue;
        };
        let supported = supported_extension(*owner);
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !path.is_file() || name.starts_with('.') || !supported.contains(&ext.as_str()) {
                continue;
            }
            let encrypted_format = matches!(owner, Format::BankSbi | Format::MutualFundsCams);
            match xfina::detect::hint::from_filename(Some(name)) {
                Some(f) if f == *owner => {
                    hinted += 1;
                    if encrypted_format {
                        encrypted_checked += 1;
                    }
                }
                // Reported by directory and extension only; a statement's
                // filename can carry account digits and a holder's name.
                Some(f) => wrong.push(format!("  {dir}/*.{ext} hinted as {f}, want {owner}")),
                None if encrypted_format => wrong.push(format!(
                    "  {dir}/*.{ext} produced no hint, and an encrypted format \
                     has nothing else to name itself with"
                )),
                None => missed += 1,
            }
        }
    }

    println!("{hinted} statements named themselves, {missed} carried no usable name");
    assert!(
        wrong.is_empty(),
        "filename hints disagree:\n{}",
        wrong.join("\n")
    );
    assert!(
        encrypted_checked > 0,
        "no encrypted statement was checked, so the case that matters is untested"
    );
}
