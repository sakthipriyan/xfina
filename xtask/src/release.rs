use std::fs;
use std::process::{exit, Command};

pub fn run_prepare(args: &[String]) {
    if args.is_empty() || !["major", "minor", "patch"].contains(&args[0].as_str()) {
        eprintln!("Usage: cargo xtask prepare-release <major|minor|patch>");
        exit(1);
    }

    let bump_type = &args[0];

    // 0. Ensure we are on the main branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if current_branch != "main" {
        eprintln!("Error: Preparing a release is only allowed starting from the 'main' branch.");
        exit(1);
    }

    // Ensure working directory is clean
    let status = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .expect("Failed to check git diff");
    if !status.success() {
        eprintln!("Error: Working directory is not clean. Commit or stash changes first.");
        exit(1);
    }

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

    let branch_name = format!("release/v{}", current_version);
    println!("Creating branch {}...", branch_name);
    run_cmd("git", &["checkout", "-b", &branch_name]);

    // Write Cargo.toml
    fs::write(cargo_path, new_cargo_content).expect("Failed to write Cargo.toml");

    // 2. Read CHANGELOG.md
    let changelog_path = "CHANGELOG.md";
    let changelog_content =
        fs::read_to_string(changelog_path).expect("Failed to read CHANGELOG.md");

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

    // 3. Sync Cargo.lock
    run_cmd("cargo", &["check"]);

    // 4. Git commands
    run_cmd("git", &["add", "Cargo.toml", "Cargo.lock", "CHANGELOG.md"]);

    let commit_msg = format!("chore(release): v{}", current_version);
    run_cmd("git", &["commit", "-m", &commit_msg]);

    println!("Successfully prepared release v{}.", current_version);
    println!("Run the following commands to create the PR:");
    println!("  git push -u origin {}", branch_name);
    println!(
        "  gh pr create --title \"{}\" --body \"Bumps version and updates changelog for release.\"",
        commit_msg
    );
    println!(
        "\nOnce the PR is merged into main, run `cargo xtask tag-release` from the main branch."
    );
}

pub fn run_tag(_args: &[String]) {
    // Ensure we are on the main branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if current_branch != "main" {
        eprintln!("Error: Tagging a release is only allowed on the 'main' branch.");
        exit(1);
    }

    // Read Cargo.toml to get the version
    let cargo_path = "Cargo.toml";
    let cargo_content = fs::read_to_string(cargo_path).expect("Failed to read Cargo.toml");

    let mut current_version = String::new();
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
            break;
        }
    }

    if current_version.is_empty() {
        eprintln!("Error: Could not find workspace.package.version in Cargo.toml");
        exit(1);
    }

    let tag_name = format!("v{}", current_version);

    // Check if tag already exists
    let status = Command::new("git")
        .args([
            "rev-parse",
            "-q",
            "--verify",
            &format!("refs/tags/{}", tag_name),
        ])
        .status()
        .expect("Failed to check git tags");

    if status.success() {
        eprintln!("Error: Tag {} already exists.", tag_name);
        exit(1);
    }

    println!("Tagging current commit as {}...", tag_name);
    run_cmd("git", &["tag", &tag_name]);

    println!("Pushing tag to origin...");
    run_cmd("git", &["push", "origin", &tag_name]);

    println!(
        "Successfully tagged and pushed {}! The publish workflow will now run.",
        tag_name
    );
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
