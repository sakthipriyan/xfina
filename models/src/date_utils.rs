use chrono::{Datelike, NaiveDate};

pub fn derive_transaction_date(stmt_date: NaiveDate, tx_day: u32, tx_month: u32) -> NaiveDate {
    let stmt_year = stmt_date.year();
    let stmt_month = stmt_date.month();

    // If the transaction month is strictly greater than the statement month, 
    // it must have occurred in the previous year.
    let tx_year = if tx_month > stmt_month {
        stmt_year - 1
    } else {
        stmt_year
    };

    NaiveDate::from_ymd_opt(tx_year, tx_month, tx_day).unwrap_or(stmt_date)
}

use chrono::{Months, Duration};

pub fn derive_statement_period(stmt_date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let prev_month = stmt_date.checked_sub_months(Months::new(1)).unwrap_or(stmt_date);
    let start_date = prev_month.checked_add_signed(Duration::days(1)).unwrap_or(prev_month);
    (start_date, stmt_date)
}
