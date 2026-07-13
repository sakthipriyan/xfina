use calamine::{Reader, open_workbook_auto, DataType};

fn main() {
    let path = "../xfina-test-data/bank-accounts/axis/Axis Bank Statement_XLS.xls";
    let mut workbook = open_workbook_auto(path).unwrap();
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        println!("Bounds: {:?}", range.get_size());
        let rows: Vec<_> = range.rows().collect();
        for (i, row) in rows.iter().enumerate().take(20) {
            let cols: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            println!("TOP {:02}: {:?}", i, cols);
        }
        let total = rows.len();
        for (i, row) in rows.iter().enumerate().skip(total.saturating_sub(20)) {
            let cols: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            println!("BOT {:02}: {:?}", i, cols);
        }
    }
}
