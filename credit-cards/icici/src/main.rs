use std::fs;
use finx_cc_icici::parse_icici_statement;

fn main() {
    let bytes = fs::read("/Users/sakthipriyan/Downloads/CCStatement_Past29-06-2026.xls").unwrap();
    use calamine::{Reader, Xlsx, open_workbook_from_rs};
    use std::io::Cursor;
    
    let mut workbook: Xlsx<_> = open_workbook_from_rs(Cursor::new(&bytes)).unwrap();
    let sheet_names = workbook.sheet_names().to_owned();
    println!("Sheets: {:?}", sheet_names);
    
    let first_sheet = &sheet_names[0];
    let range = workbook.worksheet_range(first_sheet).unwrap();
    
    for (i, row) in range.rows().enumerate().take(20) {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        println!("Row {}: {:?}", i, cells);
    }
    
    let result = parse_icici_statement(&bytes);
    match result {
        Ok(stmt) => {
            println!("{:#?}", stmt);
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
