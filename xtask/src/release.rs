//! Preparing and tagging a release.
//!
//! A release is one pull request and one tag. `prepare-release` runs on the
//! branch you are already working on, so the version bump and the changelog
//! entry ride along with the change they describe; `tag-release` then verifies
//! that `main` really does declare that version before pushing the tag that
//! publishes it.
//!
//! Entries are written per-PR under `## [Unreleased]`, as CI requires, and
//! `prepare-release` moves that block into the new version's section. It will
//! not invent one: an empty block means nobody wrote the notes, and a section
//! generated from commit subjects would hide that rather than fix it.

use std::fs;
use std::process::{exit, Command};

const CARGO_TOML: &str = "Cargo.toml";
const CHANGELOG: &str = "CHANGELOG.md";
const UNRELEASED: &str = "## [Unreleased]";

/// Paths whose contents ship to a registry. Used to spot code that landed
/// after the release was prepared and is therefore missing from its notes.
const SHIPPED_PATHS: [&str; 4] = ["src", "wasm", "python", "web"];

pub fn run_prepare(args: &[String]) {
    let mut bump = None;
    let mut own_branch = false;
    for arg in args {
        match arg.as_str() {
            "--branch" => own_branch = true,
            other if bump.is_none() => bump = Some(other.to_string()),
            other => {
                eprintln!("Error: unexpected argument '{}'", other);
                usage_prepare();
            }
        }
    }
    let Some(bump) = bump else { usage_prepare() };

    require_clean_tree();
    let branch = current_branch();

    // On main there is no pull request to attach this to, and committing the
    // bump straight to main would skip review entirely.
    if branch == "main" && !own_branch {
        eprintln!("Error: you are on 'main'. A release is prepared on the branch it ships with.");
        eprintln!("  - to add it to the work in flight: switch to that branch and re-run");
        eprintln!("  - to release what is already on main: re-run with --branch");
        exit(1);
    }

    let current = workspace_version();
    let version = next_version(&current, &bump);
    let tag = format!("v{}", version);

    if tag_exists(&tag) {
        eprintln!(
            "Error: {} already exists. That version has been released.",
            tag
        );
        exit(1);
    }

    if own_branch {
        let name = format!("release/{}", tag);
        println!("Creating branch {}...", name);
        run_cmd("git", &["checkout", "-b", &name]);
    }

    // Checked before anything is written: a refusal that left Cargo.toml
    // modified would trip the clean-tree check on the next attempt.
    require_notes_for(&version);

    println!("Bumping version: {} -> {}", current, version);
    set_workspace_version(&version);
    write_changelog_section(&version);

    // Cargo.lock records the workspace version for every member.
    run_cmd("cargo", &["check", "--quiet"]);
    run_cmd("git", &["add", CARGO_TOML, "Cargo.lock", CHANGELOG]);

    // Re-running after an edit that changed nothing -- same version, same date
    // -- is not a failure; there is simply nothing more to record.
    if git(&["diff", "--cached", "--name-only"]).is_empty() {
        println!(
            "\nNothing changed; {} is already prepared on this branch.",
            tag
        );
        return;
    }

    run_cmd(
        "git",
        &["commit", "-m", &format!("chore(release): {}", tag)],
    );

    println!("\nPrepared {}.", tag);
    println!("Edit the changelog section if the draft needs it, then amend:");
    println!("  git commit --amend");
    println!("\nOpen the pull request, and once it is merged into main:");
    println!("  cargo xtask tag-release");
}

fn usage_prepare() -> ! {
    eprintln!("Usage: cargo xtask prepare-release <major|minor|patch|X.Y.Z> [--branch]");
    eprintln!();
    eprintln!("  Run this on the branch that carries the release. It bumps the workspace");
    eprintln!("  version, drafts the changelog section from the commits since the last tag,");
    eprintln!("  and commits both.");
    eprintln!();
    eprintln!("  --branch  cut a release/vX.Y.Z branch first, for releasing what is already");
    eprintln!("            on main with no other change to carry it.");
    exit(1)
}

pub fn run_tag(_args: &[String]) {
    if current_branch() != "main" {
        eprintln!("Error: a release is tagged on 'main'.");
        exit(1);
    }
    require_clean_tree();

    // A stale local main would tag a commit that is not what was reviewed.
    run_cmd("git", &["fetch", "--quiet", "origin", "main"]);
    let local = git(&["rev-parse", "HEAD"]);
    let remote = git(&["rev-parse", "origin/main"]);
    if local != remote {
        eprintln!("Error: local main is not what origin/main points at.");
        eprintln!("  local  {}", &local[..12.min(local.len())]);
        eprintln!("  origin {}", &remote[..12.min(remote.len())]);
        eprintln!("Pull, then re-run.");
        exit(1);
    }

    let version = workspace_version();
    let tag = format!("v{}", version);

    if tag_exists(&tag) {
        eprintln!("Error: {} already exists.", tag);
        exit(1);
    }

    // The release PR is gone, so this is the only checkpoint left: main has to
    // say it is releasing this version. Without it a branch that forgot to run
    // prepare-release would tag an unbumped commit, and the pipeline would try
    // to republish a version crates.io and npm already have -- while PyPI,
    // which skips existing versions, quietly reported success.
    let notes = changelog_section(&version);
    match notes {
        None => {
            eprintln!("Error: CHANGELOG.md has no '## [{}]' section.", version);
            eprintln!("main does not declare this release. Run `cargo xtask prepare-release`");
            eprintln!("on a branch, merge it, then tag.");
            exit(1);
        }
        Some(body) if body.trim().is_empty() => {
            eprintln!(
                "Error: the '## [{}]' section in CHANGELOG.md is empty.",
                version
            );
            exit(1);
        }
        Some(_) => {}
    }

    warn_on_unreleased_drift(&version);

    println!("Tagging {} at {}...", tag, &local[..12.min(local.len())]);
    run_cmd("git", &["tag", &tag]);
    run_cmd("git", &["push", "origin", &tag]);
    println!("\nPushed {}. The publish workflow takes it from here.", tag);
}

/// Warns when code landed after the release was prepared.
///
/// Not an error: a docs-only follow-up after the bump is ordinary. But under
/// the stacked flow it is easy to merge one more branch and ship code that the
/// notes do not mention, and that is worth seeing before the tag goes out.
fn warn_on_unreleased_drift(version: &str) {
    let needle = format!("version = \"{}\"", version);
    let prep = git(&["log", "-1", "--format=%H", "-S", &needle, "--", CARGO_TOML]);
    if prep.is_empty() {
        return;
    }
    let range = format!("{}..HEAD", prep);
    let mut cmd = vec!["log", "--oneline", "--no-merges", &range, "--"];
    cmd.extend(SHIPPED_PATHS);
    let after = git(&cmd);
    if after.is_empty() {
        return;
    }
    eprintln!(
        "Warning: these landed after {} was prepared, so the notes may not cover them:",
        version
    );
    for line in after.lines() {
        eprintln!("    {}", line);
    }
    eprintln!();
}

// ---------------------------------------------------------------------------
// CHANGELOG
// ---------------------------------------------------------------------------

/// Inserts (or re-dates) the section for `version`, drafting its body from the
/// commits since the last tag when nothing has been written under
/// `## [Unreleased]`.
/// Refuses unless there is something to release with.
///
/// Either the section already exists -- a re-run, where the notes were moved
/// the first time -- or there are entries under `## [Unreleased]` to move.
/// Nothing is generated: an empty block means nobody wrote the notes, and
/// filling it from commit subjects would hide that rather than fix it.
fn require_notes_for(version: &str) {
    let content = fs::read_to_string(CHANGELOG).expect("Failed to read CHANGELOG.md");
    if content.contains(&format!("## [{}]", version)) {
        return;
    }
    let Some(at) = content.find(UNRELEASED) else {
        eprintln!("Error: could not find '{}' in CHANGELOG.md", UNRELEASED);
        exit(1);
    };
    let after = at + UNRELEASED.len();
    let end = content[after..]
        .find("\n## ")
        .map(|i| after + i)
        .unwrap_or(content.len());
    if content[after..end].trim().is_empty() {
        eprintln!(
            "Error: nothing is written under '{}' in CHANGELOG.md.",
            UNRELEASED
        );
        eprintln!();
        eprintln!("The release notes say what shipped, and only the person who made the");
        eprintln!("change can write them. Add the entries, then re-run.");
        exit(1);
    }
}

fn write_changelog_section(version: &str) {
    let content = fs::read_to_string(CHANGELOG).expect("Failed to read CHANGELOG.md");
    let today = git_date();

    // Re-running after review feedback should re-date the section, not add a
    // second one.
    if let Some(start) = content.find(&format!("## [{}]", version)) {
        let end = content[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(content.len());
        let replaced = format!("## [{}] - {}", version, today);
        let updated = format!("{}{}{}", &content[..start], replaced, &content[end..]);
        fs::write(CHANGELOG, updated).expect("Failed to write CHANGELOG.md");
        println!("Re-dated the existing [{}] section.", version);
        return;
    }

    let Some(at) = content.find(UNRELEASED) else {
        eprintln!("Error: could not find '{}' in CHANGELOG.md", UNRELEASED);
        exit(1);
    };
    let after = at + UNRELEASED.len();
    // Everything up to the next section heading belongs to Unreleased.
    let end = content[after..]
        .find("\n## ")
        .map(|i| after + i)
        .unwrap_or(content.len());

    // require_notes_for has already established that this is not empty.
    let body = content[after..end].trim().to_string();
    println!("Moving what is under [Unreleased] into [{}].", version);

    let section = format!(
        "{}\n\n## [{}] - {}\n\n{}\n",
        UNRELEASED, version, today, body
    );
    let updated = format!("{}{}{}", &content[..at], section, &content[end..]);
    fs::write(CHANGELOG, updated).expect("Failed to write CHANGELOG.md");
}

/// The body of a version's section, or `None` if it has none.
fn changelog_section(version: &str) -> Option<String> {
    let content = fs::read_to_string(CHANGELOG).ok()?;
    let start = content.find(&format!("## [{}]", version))?;
    let after = content[start..].find('\n').map(|i| start + i)?;
    let end = content[after..]
        .find("\n## ")
        .map(|i| after + i)
        .unwrap_or(content.len());
    Some(content[after..end].to_string())
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

fn next_version(current: &str, bump: &str) -> String {
    if let Some(rest) = bump.strip_prefix('v') {
        return validated(rest);
    }
    if bump.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return validated(bump);
    }

    let parts: Vec<u32> = current
        .split('.')
        .map(|p| {
            p.parse().unwrap_or_else(|_| {
                eprintln!("Error: cannot read '{}' as a version.", current);
                exit(1)
            })
        })
        .collect();
    if parts.len() != 3 {
        eprintln!("Error: cannot read '{}' as a version.", current);
        exit(1);
    }
    let (major, minor, patch) = (parts[0], parts[1], parts[2]);
    match bump {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{}.{}.0", major, minor + 1),
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        _ => usage_prepare(),
    }
}

fn validated(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
        eprintln!("Error: '{}' is not a X.Y.Z version.", version);
        exit(1);
    }
    version.to_string()
}

fn workspace_version() -> String {
    let content = fs::read_to_string(CARGO_TOML).expect("Failed to read Cargo.toml");
    let mut in_workspace_package = false;
    for line in content.lines() {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
        } else if line.starts_with('[') {
            in_workspace_package = false;
        }
        if in_workspace_package && line.starts_with("version = \"") {
            let start = line.find('"').unwrap() + 1;
            let end = line.rfind('"').unwrap();
            return line[start..end].to_string();
        }
    }
    eprintln!("Error: could not find workspace.package.version in Cargo.toml");
    exit(1)
}

fn set_workspace_version(version: &str) {
    let content = fs::read_to_string(CARGO_TOML).expect("Failed to read Cargo.toml");
    let mut out = String::with_capacity(content.len());
    let mut in_workspace_package = false;
    let mut done = false;
    for line in content.lines() {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
        } else if line.starts_with('[') {
            in_workspace_package = false;
        }
        if in_workspace_package && !done && line.starts_with("version = \"") {
            out.push_str(&format!("version = \"{}\"\n", version));
            done = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !done {
        eprintln!("Error: could not find workspace.package.version in Cargo.toml");
        exit(1);
    }
    fs::write(CARGO_TOML, out).expect("Failed to write Cargo.toml");
}

// ---------------------------------------------------------------------------
// git
// ---------------------------------------------------------------------------

fn current_branch() -> String {
    git(&["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Both the index and the working tree: `git diff --quiet` alone reports a
/// clean tree when changes are merely staged.
fn require_clean_tree() {
    if !git(&["status", "--porcelain"]).is_empty() {
        eprintln!("Error: working tree is not clean. Commit or stash first.");
        exit(1);
    }
}

fn tag_exists(tag: &str) -> bool {
    let local = Command::new("git")
        .args(["rev-parse", "-q", "--verify", &format!("refs/tags/{}", tag)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if local {
        return true;
    }
    // A tag pushed from elsewhere is just as taken.
    !git(&[
        "ls-remote",
        "--tags",
        "origin",
        &format!("refs/tags/{}", tag),
    ])
    .is_empty()
}

fn git_date() -> String {
    let out = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .expect("Failed to run date");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git(args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run git {}: {}", args.join(" "), e));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
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
