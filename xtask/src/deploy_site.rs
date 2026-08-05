use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VersionRegistry {
    latest: Option<VersionInfo>,
    unreleased: bool,
    series: Vec<SeriesInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VersionInfo {
    minor: String,
    patch: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SeriesInfo {
    minor: String,
    #[serde(rename = "latestPatch")]
    latest_patch: String,
    path: String,
}

pub fn run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cargo xtask deploy-site <--unreleased | --tag VERSION>");
        exit(1);
    }

    let mut is_unreleased = false;
    let mut tag_version = String::new();

    if args[0] == "--unreleased" {
        is_unreleased = true;
    } else if args[0] == "--tag" && args.len() > 1 {
        tag_version = args[1].clone();
        if tag_version.starts_with('v') {
            tag_version = tag_version[1..].to_string();
        }
    } else {
        eprintln!("Usage: cargo xtask deploy-site <--unreleased | --tag VERSION>");
        exit(1);
    }

    let workspace_root = env::var("CARGO_MANIFEST_DIR")
        .map(|v| PathBuf::from(v).parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| env::current_dir().unwrap());

    let worktree_dir = workspace_root.join("gh-pages-worktree");
    let web_dir = workspace_root.join("web");
    let wasm_dir = workspace_root.join("wasm");

    // 1. Cleanup stale worktrees
    println!("Cleaning up any stale git worktrees...");
    let _ = Command::new("git").current_dir(&workspace_root).args(["worktree", "remove", "--force", "gh-pages-worktree"]).status();
    let _ = Command::new("git").current_dir(&workspace_root).args(["worktree", "prune"]).status();

    // 2. Add worktree
    println!("Checking out gh-pages branch into gh-pages-worktree...");
    let status = Command::new("git")
        .current_dir(&workspace_root)
        .args(["worktree", "add", "gh-pages-worktree", "gh-pages"])
        .status()
        .expect("Failed to run git worktree add");

    if !status.success() {
        // If it fails, maybe gh-pages doesn't exist locally. Try to fetch or create orphan
        let status2 = Command::new("git")
            .current_dir(&workspace_root)
            .args(["worktree", "add", "-B", "gh-pages", "gh-pages-worktree", "origin/gh-pages"])
            .status();
        if !status2.map(|s| s.success()).unwrap_or(false) {
             eprintln!("Could not checkout gh-pages branch. Ensure it exists.");
             exit(1);
        }
    }

    // 3. Load versions.json
    let versions_path = worktree_dir.join("versions.json");
    let mut registry: VersionRegistry = if versions_path.exists() {
        let content = fs::read_to_string(&versions_path).unwrap();
        serde_json::from_str(&content).unwrap_or_else(|_| VersionRegistry {
            latest: None,
            unreleased: false,
            series: vec![],
        })
    } else {
        VersionRegistry {
            latest: None,
            unreleased: false,
            series: vec![],
        }
    };

    if is_unreleased {
        println!("Building Unreleased version...");
        run_cmd(&wasm_dir, "wasm-pack", &["build", "--target", "web"]);
        
        // Ensure dependencies like Vite are installed
        run_cmd(&web_dir, "npm", &["install"]);
        
        let mut build_cmd = Command::new("npm");
        build_cmd.current_dir(&web_dir)
            .args(["run", "build"])
            .env("VITE_APP_VERSION", "Unreleased")
            .env("VITE_DOMAIN_BASE", "/");
        run_cmd_obj(&mut build_cmd);

        let target_dir = worktree_dir.join("unreleased");
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).unwrap();
        }
        
        // Use standard copy since fs::copy only copies files
        run_cmd(&workspace_root, "cp", &["-r", "web/dist", "gh-pages-worktree/unreleased"]);
        
        registry.unreleased = true;
    } else {
        println!("Building Tagged version: {}...", tag_version);
        let parts: Vec<&str> = tag_version.split('.').collect();
        let minor_version = format!("{}.{}", parts[0], parts[1]);
        
        run_cmd(&web_dir, "npm", &["install", "--no-save", &format!("xfina-wasm@{}", tag_version)]);
        
        let mut build_cmd = Command::new("npm");
        build_cmd.current_dir(&web_dir)
            .args(["run", "build"])
            .env("VITE_APP_VERSION", &tag_version)
            .env("VITE_DOMAIN_BASE", "/");
        run_cmd_obj(&mut build_cmd);

        let target_dir = worktree_dir.join(&minor_version);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).unwrap();
        }
        
        run_cmd(&workspace_root, "cp", &["-r", "web/dist", &format!("gh-pages-worktree/{}", minor_version)]);

        // Update registry
        let mut found = false;
        for series in &mut registry.series {
            if series.minor == minor_version {
                series.latest_patch = tag_version.clone();
                found = true;
                break;
            }
        }
        if !found {
            registry.series.push(SeriesInfo {
                minor: minor_version.clone(),
                latest_patch: tag_version.clone(),
                path: format!("/{}/", minor_version),
            });
            // Sort descending by minor version
            registry.series.sort_by(|a, b| b.minor.cmp(&a.minor));
        }

        // Determine if this is the newest minor series
        let is_latest = match &registry.latest {
            Some(latest) => {
                // simple lexicographic compare works for x.y since they are padded/small, 
                // but better to parse. For now, string cmp works for 0.1 vs 0.2
                minor_version >= latest.minor
            },
            None => true
        };

        if is_latest {
            registry.latest = Some(VersionInfo {
                minor: minor_version.clone(),
                patch: tag_version.clone(),
            });

            // Mirror to root
            let index_path = worktree_dir.join("index.html");
            let assets_dir = worktree_dir.join("assets");
            
            if index_path.exists() { fs::remove_file(&index_path).unwrap(); }
            if assets_dir.exists() { fs::remove_dir_all(&assets_dir).unwrap(); }
            
            fs::copy(target_dir.join("index.html"), index_path).unwrap();
            run_cmd(&workspace_root, "cp", &["-r", format!("gh-pages-worktree/{}/assets", minor_version).as_str(), "gh-pages-worktree/assets"]);
        }
    }

    // Write versions.json
    let new_json = serde_json::to_string_pretty(&registry).unwrap();
    fs::write(versions_path, new_json).unwrap();

    // Commit and push
    println!("Checking for changes...");
    let diff_status = Command::new("git")
        .current_dir(&worktree_dir)
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to run git status");
    
    if diff_status.stdout.is_empty() {
        println!("No changes to publish.");
    } else {
        run_cmd(&worktree_dir, "git", &["add", "."]);
        run_cmd(&worktree_dir, "git", &["commit", "-m", "Deploy site update"]);
        run_cmd(&worktree_dir, "git", &["push", "origin", "gh-pages"]);
        println!("Successfully deployed to gh-pages.");
    }
    
    // Cleanup
    let _ = Command::new("git").current_dir(&workspace_root).args(["worktree", "remove", "--force", "gh-pages-worktree"]).status();
}

fn run_cmd(dir: &Path, cmd: &str, args: &[&str]) {
    println!("> {} {}", cmd, args.join(" "));
    let status = Command::new(cmd)
        .current_dir(dir)
        .args(args)
        .status()
        .expect("Failed to execute command");
        
    if !status.success() {
        eprintln!("Command failed!");
        exit(1);
    }
}

fn run_cmd_obj(cmd: &mut Command) {
    println!("> {:?}", cmd);
    let status = cmd.status().expect("Failed to execute command");
    if !status.success() {
        eprintln!("Command failed!");
        exit(1);
    }
}
