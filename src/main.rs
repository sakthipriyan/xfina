use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use xfina::models::validation::ParseResult;

fn serialize_result<T: serde::Serialize>(
    stmt: &ParseResult<T>,
    data_json: serde_json::Value,
) -> Result<String> {
    let mut root = serde_json::to_value(stmt)?;
    if let Some(obj) = root.as_object_mut() {
        obj.insert("data".to_string(), data_json);
    }
    serde_json::to_string_pretty(&root).map_err(Into::into)
}

#[derive(Parser, Debug)]
#[command(name = "xfina", about = "CLI to parse financial statements", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse a financial statement into JSON
    Parse {
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
    },
    /// Dump raw text from a PDF or XLS file for development
    Dump {
        /// The input file to dump
        file: PathBuf,

        /// Password for encrypted PDFs (if required)
        #[arg(short, long)]
        password: Option<String>,

        /// Optional output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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

fn dump_file(file: &PathBuf, password: Option<&str>, output: Option<&PathBuf>) -> Result<()> {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content = match ext.as_str() {
        "xls" | "xlsx" | "csv" => dump_spreadsheet(file)?,
        "pdf" => dump_pdf(file, password)?,
        _ => bail!("Unsupported file extension for dump: {}", ext),
    };

    if let Some(out_path) = output {
        fs::write(out_path, content)?;
        println!("Dump saved to {:?}", out_path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn dump_spreadsheet(file: &PathBuf) -> Result<String> {
    use calamine::{open_workbook_auto, Data, Reader};
    let mut workbook =
        open_workbook_auto(file).with_context(|| format!("Failed to open workbook: {:?}", file))?;

    let mut result = String::new();
    let sheets = workbook.sheet_names().to_owned();

    for sheet_name in sheets {
        result.push_str(&format!("--- Sheet: {} ---\n", sheet_name));
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows() {
                let row_str = row
                    .iter()
                    .map(|c| match c {
                        Data::String(s) => s.clone(),
                        Data::Float(f) => f.to_string(),
                        Data::Int(i) => i.to_string(),
                        Data::Bool(b) => b.to_string(),
                        Data::DateTime(_) => format!("{:?}", c),
                        Data::DateTimeIso(_) => format!("{:?}", c),
                        Data::DurationIso(_) => format!("{:?}", c),
                        Data::Error(e) => format!("{:?}", e),
                        Data::Empty => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join("\t");
                result.push_str(&row_str);
                result.push('\n');
            }
        }
    }
    Ok(result)
}

fn dump_pdf(file: &PathBuf, password: Option<&str>) -> Result<String> {
    let bytes = fs::read(file)?;
    let mut doc = pdf_extract::Document::load_mem(&bytes)
        .map_err(|e| anyhow::anyhow!("Failed to load PDF: {:?}", e))?;

    if let Some(pw) = password {
        doc.decrypt(pw)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt PDF: {:?}", e))?;
    } else if doc.is_encrypted() {
        bail!("PDF is encrypted, password required");
    }

    let mut out = String::new();
    {
        let mut plain_text_out = pdf_extract::PlainTextOutput::new(&mut out);
        pdf_extract::output_doc(&doc, &mut plain_text_out)
            .map_err(|e| anyhow::anyhow!("Failed to extract PDF: {:?}", e))?;
    }
    Ok(out)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dump {
            file,
            password,
            output,
        } => {
            dump_file(&file, password.as_deref(), output.as_ref())?;
        }
        Commands::Parse {
            category,
            institution,
            file,
            password,
            output,
            format,
        } => {
            let file_bytes =
                fs::read(&file).with_context(|| format!("Failed to read file: {:?}", file))?;

            let file_name = file.file_stem().and_then(|s| s.to_str());

            let output_path = output.unwrap_or_else(|| {
                let mut path = file.clone();
                path.set_extension("json");
                path
            });

            let is_rebit = format.to_lowercase() == "rebit";

            let modified_timestamp = std::fs::metadata(&file)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);

            let req = xfina::models::request::ParseRequest::new(&file_bytes)
                .with_password(password.as_deref())
                .with_filename(file_name)
                .with_modified_timestamp(modified_timestamp);

            let json_output = match (category, institution) {
                (Category::BankAccount, Institution::Hdfc) => {
                    let res = xfina::bank_accounts::hdfc::parse_hdfc_bank_statement(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::BankAccount, Institution::Icici) => {
                    let res = xfina::bank_accounts::icici::parse_icici_bank_statement(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::BankAccount, Institution::Sbi) => {
                    let res = xfina::bank_accounts::sbi::parse_sbi_bank_statement(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::BankAccount, Institution::Bob) => {
                    let res = xfina::bank_accounts::bob::parse_bob_xls(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::BankAccount, Institution::Axis) => {
                    let res = xfina::bank_accounts::axis::parse_axis_bank_statement(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::CreditCard, Institution::Hdfc) => {
                    let res = xfina::credit_cards::hdfc::parse_hdfc_statement(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::CreditCard, Institution::Icici) => {
                    let res = xfina::credit_cards::icici::parse_icici_statement(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::MutualFund, Institution::Cams) => {
                    let res = xfina::mutual_funds::cams::parse_cams_pdf(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                (Category::IntlStocks, Institution::Ibkr) => {
                    let res = xfina::intl_stocks::ibkr::parse_ibkr_csv(req.clone())?;
                    let json = if is_rebit {
                        res.data.to_rebit_json()
                    } else {
                        res.data.to_xfina_json()
                    };
                    serialize_result(&res, json)?
                }
                _ => bail!("Unsupported combination of category and institution"),
            };

            fs::write(&output_path, json_output)?;
            println!("Successfully parsed to {:?}", output_path);
        }
    }

    Ok(())
}
