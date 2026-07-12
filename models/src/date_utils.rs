use chrono::{Datelike, NaiveDate, DateTime, Utc, TimeZone, Months, Duration};

/// Derives the full transaction date (with year) from partial date info.
/// Returns `DateTime<Utc>` at midnight IST, converted to UTC.
pub fn derive_transaction_date(stmt_date: NaiveDate, tx_day: u32, tx_month: u32) -> DateTime<Utc> {
    let stmt_year = stmt_date.year();
    let stmt_month = stmt_date.month();

    // If the transaction month is strictly greater than the statement month,
    // it must have occurred in the previous year.
    let tx_year = if tx_month > stmt_month {
        stmt_year - 1
    } else {
        stmt_year
    };

    let naive = NaiveDate::from_ymd_opt(tx_year, tx_month, tx_day)
        .unwrap_or(stmt_date);
    let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
    let ndt = naive.and_hms_opt(0, 0, 0).unwrap();
    chrono::TimeZone::from_local_datetime(&ist_offset, &ndt)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| naive.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// Derives the statement period (start, end) as `DateTime<Utc>` at midnight IST.
pub fn derive_statement_period(stmt_date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let prev_month = stmt_date.checked_sub_months(Months::new(1)).unwrap_or(stmt_date);
    let start_date = prev_month.checked_add_signed(Duration::days(1)).unwrap_or(prev_month);
    (start_date, stmt_date)
}
