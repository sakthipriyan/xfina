use calamine::{Reader, open_workbook_auto};

fn main() {
    let mut workbook = open_workbook_auto("../../../financial-extract-test-data/bank-accounts/icici/raw/OpTransactionHistory05-07-2026.xls").unwrap();
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        for (i, row) in range.rows().enumerate().take(30) {
            let cols: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            println!("{:02}: {:?}", i, cols);
        }
    }
}
