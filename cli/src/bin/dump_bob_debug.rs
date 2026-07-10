use calamine::{Reader, open_workbook_auto};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file.xls>", args[0]);
        return;
    }
    let mut workbook = open_workbook_auto(&args[1]).unwrap();
    let sheet_names = workbook.sheet_names().to_owned();
    if let Ok(sheet) = workbook.worksheet_range(&sheet_names[0]) {
        for (i, row) in sheet.rows().enumerate().take(30) {
            let row_vec: Vec<String> = row.iter().map(|c| c.to_string().replace("\u{0}", "").trim().to_string()).collect();
            println!("Row {}: {:?}", i, row_vec);
        }
    }
}
