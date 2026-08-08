use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// -----------------------------------------------------------------------------
// ParseResult — top-level envelope returned by every Xfina parser
// -----------------------------------------------------------------------------

/// Top-level envelope returned by every Xfina parser.
///
/// Separates the parse-layer metadata (`validation`) from the financial
/// data model (`data`), keeping the ReBIT/AA-adjacent account structs clean.
///
/// Parsing **always** succeeds if the document can be read; a validation
/// mismatch is surfaced in `validation.overall` rather than as an `Err`.
/// Use `Err` only for genuine parse failures (wrong format, corrupt file,
/// encrypted PDF without a password).
///
/// # Example
///
/// ```no_run
/// use xfina::bank_accounts::hdfc::parse_hdfc_bank_statement;
/// use xfina::models::validation::ValidationStatus;
///
/// let bytes = std::fs::read("statement.xls").unwrap();
/// use xfina::models::request::ParseRequest;
///
/// let req = ParseRequest::new(&bytes);
/// let result = parse_hdfc_bank_statement(req).unwrap();
///
/// match result.validation.overall {
///     ValidationStatus::Passed  => println!("✓ All checks passed"),
///     ValidationStatus::Warning => println!("⚠ Minor discrepancy"),
///     ValidationStatus::Failed  => println!("✗ Declared totals mismatch"),
/// }
///
/// // The parsed data is always present regardless of validation status
/// if let Some(profile) = result.data.profile {
///     println!("Account Name: {}", profile.holders.holder[0].name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult<T> {
    /// The parsed financial account (e.g. `DepositAccount`, `CreditCardAccount`).
    /// Always present — a validation failure does **not** prevent data from being returned.
    pub data: T,
    /// Two-level validation report computed after parsing completes.
    pub validation: ValidationReport,
}

// -----------------------------------------------------------------------------
// ValidationReport
// -----------------------------------------------------------------------------

/// Non-blocking two-level validation report.
///
/// Both levels are independently reported — a parse can pass `summary_level`
/// but fail `row_level` (e.g. two transactions that cancel each other), or
/// vice versa.
///
/// `overall` is computed from both levels after all checks run:
/// - A `Declared`-source summary failure → `Failed`
/// - Any other failure → `Warning`
/// - All checks pass → `Passed`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Rolled-up status computed from both levels.
    pub overall: ValidationStatus,
    /// Level 1: row-by-row running balance reconciliation.
    pub row_level: RowValidation,
    /// Level 2: declared or derived summary cross-checks.
    pub summary_level: SummaryValidation,
}

impl ValidationReport {
    /// Compute and set `overall` from the current state of both levels.
    /// Call this after all checks have been added.
    pub fn finalize(&mut self) {
        let has_declared_failure = self.summary_level.checks.iter().any(|c| {
            !c.passed && c.source == SummarySource::Declared
        });
        let all_passed = self.row_level.passed && self.summary_level.passed;

        self.overall = match (all_passed, has_declared_failure) {
            (true, _)  => ValidationStatus::Passed,
            (_, true)  => ValidationStatus::Failed,
            _          => ValidationStatus::Warning,
        };
    }

    /// Convenience: create an empty report (no checks run yet).
    pub fn empty() -> Self {
        Self {
            overall: ValidationStatus::Passed,
            row_level: RowValidation {
                passed: true,
                checked_rows: 0,
                failed_rows: Vec::new(),
            },
            summary_level: SummaryValidation {
                passed: true,
                checks: Vec::new(),
            },
        }
    }
}

/// Overall validation status, suitable for UI badge display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// All checks passed — show a green ✓ badge.
    Passed,
    /// Some checks failed, but all failures were on `Derived` sources —
    /// show a yellow ⚠ badge (minor discrepancy, may be rounding).
    Warning,
    /// At least one `Declared`-source check failed — show a red ✗ badge.
    /// The institution printed a number that our transactions cannot reproduce.
    Failed,
}

// -----------------------------------------------------------------------------
// Level 1 — Row-by-Row
// -----------------------------------------------------------------------------

/// Level 1 validation: row-by-row running balance reconciliation.
///
/// For each consecutive transaction pair, checks that:
/// `txn[n].balance == txn[n-1].balance ± txn[n].amount`
///
/// Only non-passing rows are included in `failed_rows` to keep the
/// serialized output compact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowValidation {
    /// `true` when `failed_rows` is empty.
    pub passed: bool,
    /// Total number of row pairs checked (0 when no running balance is available,
    /// e.g. credit card statements).
    pub checked_rows: usize,
    /// Details of rows where the computed balance diverged from the printed balance.
    /// Empty when `passed` is `true`.
    pub failed_rows: Vec<RowCheckFailure>,
}

/// A single row where the running balance check failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowCheckFailure {
    /// 0-based index of the transaction in the statement.
    pub row_index: usize,
    /// Narration of the offending transaction, for human identification.
    pub narration: String,
    /// Balance computed from `previous_balance ± amount`.
    pub expected_balance: Decimal,
    /// Balance actually printed in the statement.
    pub actual_balance: Decimal,
    /// `expected_balance - actual_balance`.
    pub delta: Decimal,
}

// -----------------------------------------------------------------------------
// Level 2 — Summary
// -----------------------------------------------------------------------------

/// Level 2 validation: summary / totals cross-checks.
///
/// Each check compares a declared or derived figure against the value
/// computed from the parsed transactions. `passed` is `true` when every
/// individual check passed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryValidation {
    /// `true` when every check in `checks` passed.
    pub passed: bool,
    /// Individual cross-checks. May be empty for parsers with no summary data.
    pub checks: Vec<SummaryCheck>,
}

/// A single summary cross-check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryCheck {
    /// Machine-readable name, e.g. `"total_credits_match"`, `"closing_balance_match"`.
    pub name: String,
    /// `true` when `declared == computed` (or within tolerance for unit checks).
    pub passed: bool,
    /// Strength of evidence for this check.
    pub source: SummarySource,
    /// Value asserted by the source document. `None` for `Derived` checks where
    /// only the computed value exists.
    #[serde(
        with = "rust_decimal::serde::float_option",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub declared: Option<Decimal>,
    /// Value derived from our parsed transactions.
    #[serde(with = "rust_decimal::serde::float")]
    pub computed: Decimal,
    /// `declared - computed`. `None` when the check passed (no discrepancy).
    #[serde(
        with = "rust_decimal::serde::float_option",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub delta: Option<Decimal>,
    /// Human-readable context, e.g. `"fund: HDFC Flexi Cap"` or `"amc: HDFC AMC"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl SummaryCheck {
    /// Build a check comparing a declared value to a computed one.
    pub fn declared(name: &str, declared: Decimal, computed: Decimal, note: Option<String>) -> Self {
        // Use a 1.0 tolerance for checking to account for floating-point rounding or bank minor adjustments
        let delta_val = declared - computed;
        let passed = delta_val.abs() < Decimal::from_str("1.0").unwrap_or_default();
        let delta = if passed { None } else { Some(delta_val) };
        Self {
            name: name.to_string(),
            passed,
            source: SummarySource::Declared,
            declared: Some(declared),
            computed,
            delta,
            note,
        }
    }

    /// Build a check comparing two computed values (no declared figure available).
    pub fn derived(name: &str, expected: Decimal, computed: Decimal, note: Option<String>) -> Self {
        // Use a 1.0 tolerance for checking to account for floating-point rounding or bank minor adjustments
        let delta_val = expected - computed;
        let passed = delta_val.abs() < Decimal::from_str("1.0").unwrap_or_default();
        let delta = if passed { None } else { Some(delta_val) };
        Self {
            name: name.to_string(),
            passed,
            source: SummarySource::Derived,
            declared: None,
            computed,
            delta,
            note,
        }
    }
}

/// Strength of evidence for a [`SummaryCheck`].
///
/// Affects how [`ValidationReport::overall`] is computed:
/// - A `Declared` check failure → [`ValidationStatus::Failed`]
/// - A `Derived` check failure  → [`ValidationStatus::Warning`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummarySource {
    /// The institution printed this number in the statement.
    /// Failing this check is a strong signal of a parsing bug or data corruption.
    Declared,
    /// This value was inferred from opening/closing balance arithmetic.
    /// Failing this check may indicate rounding differences or a missing transaction.
    Derived,
}

// -----------------------------------------------------------------------------
// Helper — run row-level check on a slice of (type, amount, balance, narration)
// -----------------------------------------------------------------------------

/// Run a row-by-row running balance check given an opening balance and a list
/// of `(is_credit, amount, printed_balance, narration)` tuples.
///
/// Returns a [`RowValidation`] that can be placed directly in a [`ValidationReport`].
pub fn check_row_balances(
    opening_balance: Decimal,
    rows: &[(bool, Decimal, Decimal, String)], // (is_credit, amount, printed_balance, narration)
) -> RowValidation {
    let mut running = opening_balance;
    let mut failed_rows = Vec::new();

    for (idx, (is_credit, amount, printed, narration)) in rows.iter().enumerate() {
        if *is_credit {
            running += amount;
        } else {
            running -= amount;
        }
        if running != *printed {
            failed_rows.push(RowCheckFailure {
                row_index: idx,
                narration: narration.clone(),
                expected_balance: running,
                actual_balance: *printed,
                delta: running - printed,
            });
            // Resync so subsequent rows are checked against reality, not a
            // compounding error.
            running = *printed;
        }
    }

    let passed = failed_rows.is_empty();
    RowValidation {
        passed,
        checked_rows: rows.len(),
        failed_rows,
    }
}
