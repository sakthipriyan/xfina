use clap::{Parser, ValueEnum};
use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::fs;

// We use explicit paths to xfina:: modules below.

#[derive(Parser, Debug)]
#[command(name = "xfina", about = "CLI to parse financial statements", version)]
struct Cli {
    /// The category of the financial statement
    #[arg(value_enum)]
    category: Category,

    /// The institution that generated the statement
    #[arg(value_enum)]
    institution: Institution,

    /// The input file to parse
    file: PathBuf,

    /// Password for encrypted PDFs (if required)
    #[arg(short, long)]
    password: Option<String>,

    /// Optional output file path (defaults to <input_file_stem>.json in the same directory)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Optional format (rebit or xfina)
    #[arg(short, long, default_value = "xfina")]
    format: String,
}

#[derive(ValueEnum, Clone, Debug)]
enum Category {
    BankAccount,
    CreditCard,
    MutualFund,
    IntlStocks,
}

#[derive(ValueEnum, Clone, Debug)]
enum Institution {
    Hdfc,
    Icici,
    Sbi,
    Bob,
    Axis,
    Cams,
    Ibkr,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let file_bytes = fs::read(&cli.file)
        .with_context(|| format!("Failed to read file: {:?}", cli.file))?;
    
    // We pass the filename to some parsers for metadata context
    let file_name = cli.file.file_stem().and_then(|s| s.to_str());
        
    let output_path = cli.output.unwrap_or_else(|| {
        let mut path = cli.file.clone();
        path.set_extension("json");
        path
    });

    let is_rebit = cli.format.to_lowercase() == "rebit";

    let json_output = match (cli.category, cli.institution) {
        (Category::BankAccount, Institution::Hdfc) => {
            let res = xfina::bank_accounts::hdfc::parse_hdfc_bank_statement(&file_bytes, cli.password.as_deref())
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::BankAccount, Institution::Icici) => {
            let res = xfina::bank_accounts::icici::parse_icici_bank_statement(&file_bytes, file_name)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::BankAccount, Institution::Sbi) => {
            let res = xfina::bank_accounts::sbi::parse_sbi_bank_statement(&file_bytes, cli.password.as_deref(), file_name)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::BankAccount, Institution::Bob) => {
            let res = xfina::bank_accounts::bob::parse_bob_xls(&file_bytes)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::BankAccount, Institution::Axis) => {
            let res = xfina::bank_accounts::axis::parse_axis_bank_statement(&file_bytes, file_name)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::CreditCard, Institution::Hdfc) => {
            let content = String::from_utf8(file_bytes.clone())
                .context("HDFC Credit Card requires valid UTF-8 text (CSV/TXT)")?;
            let res = xfina::credit_cards::hdfc::parse_hdfc_statement(&content, file_name)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::CreditCard, Institution::Icici) => {
            let res = xfina::credit_cards::icici::parse_icici_statement(&file_bytes, file_name)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::MutualFund, Institution::Cams) => {
            let res = xfina::mutual_funds::cams::parse_cams_pdf(&file_bytes, cli.password.as_deref(), file_name)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (Category::IntlStocks, Institution::Ibkr) => {
            let content = String::from_utf8(file_bytes.clone())
                .context("IBKR requires valid UTF-8 text (CSV)")?;
            let res = xfina::intl_stocks::ibkr::parse_ibkr_csv(&content)
                .map_err(|e| anyhow::anyhow!(e))?;
            let json = if is_rebit { res.to_rebit_json() } else { res.to_xfina_json() };
            serde_json::to_string_pretty(&json)?
        },
        (cat, inst) => {
            bail!("Unsupported combination: Category '{:?}' with Institution '{:?}'", cat, inst);
        }
    };

    fs::write(&output_path, json_output)
        .with_context(|| format!("Failed to write output to {:?}", output_path))?;
        
    println!("Successfully parsed and exported to {:?}", output_path);

    Ok(())
}
