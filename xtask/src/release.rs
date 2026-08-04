use std::fs;
use std::process::{Command, exit};

pub fn run(args: &[String]) {
    if args.is_empty() || !["major", "minor", "patch"].contains(&args[0].as_str()) {
        eprintln!("Usage: cargo xtask release <major|minor|patch>");
        exit(1);
    }

    let bump_type = &args[0];

    // 1. Read Cargo.toml
    let cargo_path = "Cargo.toml";
    let cargo_content = fs::read_to_string(cargo_path).expect("Failed to read Cargo.toml");

    let mut current_version = String::new();
    let mut new_cargo_content = String::new();

    let mut in_workspace_package = false;
    for line in cargo_content.lines() {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
        } else if line.starts_with('[') {
            in_workspace_package = false;
        }

        if in_workspace_package && line.starts_with("version = \"") {
            let start = line.find('"').unwrap() + 1;
            let end = line.rfind('"').unwrap();
            current_version = line[start..end].to_string();

            let parts: Vec<&str> = current_version.split('.').collect();
            let mut major: u32 = parts[0].parse().unwrap();
            let mut minor: u32 = parts[1].parse().unwrap();
            let mut patch: u32 = parts[2].parse().unwrap();

            match bump_type.as_str() {
                "major" => {
                    major += 1;
                    minor = 0;
                    patch = 0;
                }
                "minor" => {
                    minor += 1;
                    patch = 0;
                }
                "patch" => {
                    patch += 1;
                }
                _ => unreachable!(),
            }

            let new_version = format!("{}.{}.{}", major, minor, patch);
            println!("Bumping version: {} -> {}", current_version, new_version);
            new_cargo_content.push_str(&format!("version = \"{}\"\n", new_version));
            current_version = new_version; // save for changelog
        } else {
            new_cargo_content.push_str(line);
            new_cargo_content.push('\n');
        }
    }

    if current_version.is_empty() {
        eprintln!("Error: Could not find workspace.package.version in Cargo.toml");
        exit(1);
    }

    // Write Cargo.toml
    fs::write(cargo_path, new_cargo_content).expect("Failed to write Cargo.toml");

    // 2. Read CHANGELOG.md
    let changelog_path = "CHANGELOG.md";
    let changelog_content = fs::read_to_string(changelog_path).expect("Failed to read CHANGELOG.md");
    
    // Get current date YYYY-MM-DD
    let output = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .expect("Failed to execute date command");
    let date_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let unreleased_header = "## [Unreleased]";
    let new_header = format!("## [Unreleased]\n\n## [{}] - {}", current_version, date_str);
    
    if !changelog_content.contains(unreleased_header) {
        eprintln!("Error: Could not find '## [Unreleased]' in CHANGELOG.md");
        exit(1);
    }
    
    let new_changelog_content = changelog_content.replacen(unreleased_header, &new_header, 1);
    fs::write(changelog_path, new_changelog_content).expect("Failed to write CHANGELOG.md");

    // 3. Git commands
    run_cmd("git", &["add", "Cargo.toml", "CHANGELOG.md"]);
    
    let commit_msg = format!("chore(release): v{}", current_version);
    run_cmd("git", &["commit", "-m", &commit_msg]);
    
    let tag_name = format!("v{}", current_version);
    run_cmd("git", &["tag", &tag_name]);
    
    println!("Successfully bumped to {}. Ready to push!", current_version);
    println!("Run: git push origin main --tags");
}

fn run_cmd(cmd: &str, args: &[&str]) {
    println!("> {} {}", cmd, args.join(" "));
    let status = Command::new(cmd)
        .args(args)
        .status()
        .expect("Failed to execute command");
        
    if !status.success() {
        eprintln!("Command failed!");
        exit(1);
    }
}
