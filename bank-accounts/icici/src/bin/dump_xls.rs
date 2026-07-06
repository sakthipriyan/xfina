use calamine::{Reader, open_workbook, Xlsx};

fn main() {
    let mut workbook: Xlsx<_> = open_workbook("../../../financial-extract-test-data/credit-cards/icici/CCStatement_Past29-06-2026.xls").unwrap();
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        for (i, row) in range.rows().enumerate().take(50) {
            let cols: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            println!("ICICI CC {:02}: {:?}", i, cols);
        }
    }
}
