use clap::Parser;
use std::fs;
use financial_extract_cams::parser::extract_spatial_pages;
use financial_extract_cams::layout::group_into_lines;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    filepath: String,
    #[arg(short, long)]
    password: Option<String>,
}

fn main() {
    let args = Args::parse();
    let bytes = fs::read(&args.filepath).unwrap();
    let pages = extract_spatial_pages(&bytes, args.password.as_deref()).unwrap();
    
    for (i, page) in pages.iter().enumerate() {
        println!("--- PAGE {} ---", i + 1);
        let lines = group_into_lines(page, 2.0);
        for line in lines {
            println!("Y: {:.2} | TEXT: '{}'", line.baseline, line.text);
        }
    }
}
