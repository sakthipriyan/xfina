use calamine::{Reader, open_workbook_auto};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-excel-file>", args[0]);
        std::process::exit(1);
    }
    
    let path = &args[1];
    
    let mut workbook = match open_workbook_auto(path) {
        Ok(wb) => wb,
        Err(e) => {
            eprintln!("Failed to open workbook: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let sheet_names = workbook.sheet_names().to_owned();
    if sheet_names.is_empty() {
        eprintln!("No sheets found in the workbook");
        std::process::exit(1);
    }
    
    let sheet_name = &sheet_names[0];
    println!("Dumping first sheet: {}", sheet_name);
    
    if let Ok(range) = workbook.worksheet_range(sheet_name) {
        for (i, row) in range.rows().enumerate() {
            let cols: Vec<String> = row.iter().map(|c| c.to_string().replace("\u{0}", "").trim().to_string()).collect();
            println!("Row {:02}: {:?}", i, cols);
        }
    } else {
        eprintln!("Failed to read sheet {}", sheet_name);
        std::process::exit(1);
    }
}
