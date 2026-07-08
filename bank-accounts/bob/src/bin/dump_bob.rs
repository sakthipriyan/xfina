use calamine::{Reader, open_workbook_auto, Data};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return;
    }
    
    let path = &args[1];
    let mut workbook = open_workbook_auto(path).unwrap();
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names.first().unwrap();
    
    if let Ok(range) = workbook.worksheet_range(sheet_name) {
        for (i, row) in range.rows().enumerate() {
            let cols: Vec<String> = row.iter().map(|c| c.to_string().replace("\u{0}", "").trim().to_string()).collect();
            println!("Row {:02}: {:?}", i, cols);
        }
    }
}
