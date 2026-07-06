use calamine::{Reader, open_workbook_auto};

fn main() {
    let mut workbook = open_workbook_auto("../../../financial-extract-test-data/bank-accounts/icici/raw/OpTransactionHistory05-07-2026.xls").unwrap();
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let count = range.rows().count();
        for (i, row) in range.rows().enumerate().skip(count.saturating_sub(20)) {
            let cols: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            println!("ICICI {:02}: {:?}", i, cols);
        }
    }
    
    let mut workbook2 = open_workbook_auto("../../../financial-extract-test-data/bank-accounts/hdfc/raw/Acct_Statement_XXXXXXXX2144_05072026.xls").unwrap();
    if let Some(Ok(range)) = workbook2.worksheet_range_at(0) {
        let count = range.rows().count();
        for (i, row) in range.rows().enumerate().skip(count.saturating_sub(20)) {
            let cols: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            println!("HDFC {:02}: {:?}", i, cols);
        }
    }
}
