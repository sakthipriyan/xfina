use std::env;
use std::process::exit;

mod deploy_site;
mod release;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <prepare-release|tag-release|deploy-site> [args...]");
        exit(1);
    }

    match args[1].as_str() {
        "prepare-release" => release::run_prepare(&args[2..]),
        "tag-release" => release::run_tag(&args[2..]),
        "deploy-site" => deploy_site::run(&args[2..]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Usage: cargo xtask <prepare-release|tag-release|deploy-site> [args...]");
            exit(1);
        }
    }
}
