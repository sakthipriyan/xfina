use clap::Parser;
use finx_mf_cams::parse_cams_pdf;
use std::fs;

/// CLI tool to test CAMS PDF extraction
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the CAS PDF file
    #[arg(short, long)]
    filepath: String,

    /// Password for the CAS PDF file
    #[arg(short, long)]
    password: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("Reading file: {}", args.filepath);
    let bytes = match fs::read(&args.filepath) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    println!("Parsing PDF...");
    match parse_cams_pdf(&bytes, args.password.as_deref()) {
        Ok(portfolio) => {
            let json = serde_json::to_string_pretty(&portfolio).unwrap();
            println!("Parsed Portfolio:\n{}", json);
        }
        Err(e) => {
            eprintln!("Error parsing CAS PDF: {}", e);
            std::process::exit(1);
        }
    }
}
