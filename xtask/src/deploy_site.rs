use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

/// The directory the build of `main` is published to, and the key that names it
/// in the registry and the version dropdown. Released series are always `X.Y`,
/// so the two namespaces cannot collide.
const UNRELEASED: &str = "unreleased";

/// Every directory published on gh-pages, in the order the version dropdown
/// shows them: released series newest first, then the unreleased build last.
/// Position carries the meaning that separate `latest` and `unreleased` keys
/// used to -- `series[0]` is the latest release -- so there is one list to fix
/// when something in it is wrong.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct VersionRegistry {
    series: Vec<SeriesInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SeriesInfo {
    /// Identifies the entry, and is the value the version dropdown selects by.
    /// The unreleased build uses the literal `unreleased`, so one string names
    /// a site in both places.
    minor: String,
    /// Absent on the unreleased entry, which has no patch version.
    #[serde(
        rename = "latestPatch",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    latest_patch: Option<String>,
    path: String,
    /// The commit this directory was built from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
}

impl SeriesInfo {
    /// `minor` is what identifies an entry, so it is also what says whether the
    /// entry is the unreleased build. A separate flag would be a second copy of
    /// the same fact, free to disagree with it.
    fn is_unreleased(&self) -> bool {
        self.minor == UNRELEASED
    }
}

/// Released series newest first, the unreleased entry pinned last.
fn sort_series(registry: &mut VersionRegistry) {
    registry.series.sort_by(|a, b| {
        a.is_unreleased()
            .cmp(&b.is_unreleased())
            .then_with(|| b.minor.cmp(&a.minor))
    });
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
    let _ = Command::new("git")
        .current_dir(&workspace_root)
        .args(["worktree", "remove", "--force", "gh-pages-worktree"])
        .status();
    let _ = Command::new("git")
        .current_dir(&workspace_root)
        .args(["worktree", "prune"])
        .status();

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
            .args([
                "worktree",
                "add",
                "-B",
                "gh-pages",
                "gh-pages-worktree",
                "origin/gh-pages",
            ])
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
        serde_json::from_str(&content).unwrap_or_else(|_| VersionRegistry { series: vec![] })
    } else {
        VersionRegistry { series: vec![] }
    };

    // The commit the built directory came from. The web app's header badge
    // links to it, so it has to be one that outlives the build: a tag resolves
    // to the commit the tag names, which survives a squash-merge, rather than
    // whatever HEAD the runner happened to be sitting on.
    let source_commit = if is_unreleased {
        resolve_commit(&workspace_root, "HEAD")
    } else {
        resolve_commit(&workspace_root, &format!("v{}", tag_version))
            .or_else(|| resolve_commit(&workspace_root, "HEAD"))
    };
    match &source_commit {
        Some(commit) => println!("Source commit: {}", commit),
        None => eprintln!(
            "Warning: could not resolve the source commit; the registry will not record one."
        ),
    }

    if is_unreleased {
        println!("Building Unreleased version...");
        run_cmd(&wasm_dir, "wasm-pack", &["build", "--target", "web"]);

        // Ensure dependencies like Vite are installed
        run_cmd(&web_dir, "npm", &["install"]);

        let mut build_cmd = Command::new("npm");
        build_cmd
            .current_dir(&web_dir)
            .args(["run", "build"])
            .env("VITE_APP_VERSION", "Unreleased")
            .env("VITE_DOMAIN_BASE", "/");
        if let Some(commit) = &source_commit {
            build_cmd.env("VITE_COMMIT_HASH", commit);
        }
        run_cmd_obj(&mut build_cmd);

        let target_dir = worktree_dir.join("unreleased");
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).unwrap();
        }

        // Use standard copy since fs::copy only copies files
        run_cmd(
            &workspace_root,
            "cp",
            &["-r", "web/dist", "gh-pages-worktree/unreleased"],
        );

        upsert_series(
            &mut registry,
            SeriesInfo {
                minor: UNRELEASED.to_string(),
                latest_patch: None,
                path: format!("/{}/", UNRELEASED),
                commit: source_commit.clone(),
            },
        );
    } else {
        println!("Building Tagged version: {}...", tag_version);
        let parts: Vec<&str> = tag_version.split('.').collect();
        let minor_version = format!("{}.{}", parts[0], parts[1]);

        run_cmd(
            &web_dir,
            "npm",
            &[
                "install",
                "--no-save",
                &format!("xfina-wasm@{}", tag_version),
            ],
        );

        let mut build_cmd = Command::new("npm");
        build_cmd
            .current_dir(&web_dir)
            .args(["run", "build"])
            .env("VITE_APP_VERSION", &tag_version)
            .env("VITE_DOMAIN_BASE", "/");
        if let Some(commit) = &source_commit {
            build_cmd.env("VITE_COMMIT_HASH", commit);
        }
        run_cmd_obj(&mut build_cmd);

        let target_dir = worktree_dir.join(&minor_version);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).unwrap();
        }

        run_cmd(
            &workspace_root,
            "cp",
            &[
                "-r",
                "web/dist",
                &format!("gh-pages-worktree/{}", minor_version),
            ],
        );

        upsert_series(
            &mut registry,
            SeriesInfo {
                minor: minor_version.clone(),
                latest_patch: Some(tag_version.clone()),
                path: format!("/{}/", minor_version),
                commit: source_commit.clone(),
            },
        );

        // The newest release is whatever sorts to the front, so the list says
        // which series the root mirrors rather than a separate key repeating it.
        let is_latest = registry
            .series
            .first()
            .is_some_and(|series| series.minor == minor_version);

        if is_latest {
            // Mirror to root
            let index_path = worktree_dir.join("index.html");
            let assets_dir = worktree_dir.join("assets");

            if index_path.exists() {
                fs::remove_file(&index_path).unwrap();
            }
            if assets_dir.exists() {
                fs::remove_dir_all(&assets_dir).unwrap();
            }

            fs::copy(target_dir.join("index.html"), index_path).unwrap();
            run_cmd(
                &workspace_root,
                "cp",
                &[
                    "-r",
                    format!("gh-pages-worktree/{}/assets", minor_version).as_str(),
                    "gh-pages-worktree/assets",
                ],
            );
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
        run_cmd(
            &worktree_dir,
            "git",
            &["commit", "-m", "Deploy site update"],
        );
        run_cmd(&worktree_dir, "git", &["push", "origin", "gh-pages"]);
        println!("Successfully deployed to gh-pages.");
    }

    // Cleanup
    let _ = Command::new("git")
        .current_dir(&workspace_root)
        .args(["worktree", "remove", "--force", "gh-pages-worktree"])
        .status();
}

/// Replace the entry for `entry.minor`, or add it, then restore the ordering.
fn upsert_series(registry: &mut VersionRegistry, entry: SeriesInfo) {
    match registry
        .series
        .iter_mut()
        .find(|series| series.minor == entry.minor)
    {
        Some(existing) => *existing = entry,
        None => registry.series.push(entry),
    }
    sort_series(registry);
}

/// Resolve a revision to its full commit SHA, or `None` if git cannot.
fn resolve_commit(dir: &Path, rev: &str) -> Option<String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(["rev-list", "-n", "1", rev])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn released(minor: &str, patch: &str, commit: &str) -> SeriesInfo {
        SeriesInfo {
            minor: minor.to_string(),
            latest_patch: Some(patch.to_string()),
            path: format!("/{}/", minor),
            commit: Some(commit.to_string()),
        }
    }

    fn unreleased(commit: &str) -> SeriesInfo {
        SeriesInfo {
            minor: UNRELEASED.to_string(),
            latest_patch: None,
            path: format!("/{}/", UNRELEASED),
            commit: Some(commit.to_string()),
        }
    }

    // The whole point of the flat list: position is the meaning. `series[0]` is
    // the latest release, so the tagged deploy reads it to decide whether to
    // mirror the build to the root.
    #[test]
    fn latest_release_sorts_first_and_unreleased_last() {
        let mut registry = VersionRegistry {
            series: vec![unreleased("aaa"), released("0.2", "0.2.4", "bbb")],
        };

        upsert_series(&mut registry, released("0.4", "0.4.1", "ccc"));
        upsert_series(&mut registry, released("0.3", "0.3.0", "ddd"));

        let order: Vec<&str> = registry.series.iter().map(|s| s.minor.as_str()).collect();
        assert_eq!(order, ["0.4", "0.3", "0.2", UNRELEASED]);
    }

    // A tag on an existing series replaces that entry rather than adding a
    // second one, and the unreleased entry is upserted the same way on every
    // merge to main.
    #[test]
    fn upsert_replaces_an_existing_entry() {
        let mut registry = VersionRegistry {
            series: vec![released("0.4", "0.4.0", "old"), unreleased("stale")],
        };

        upsert_series(&mut registry, released("0.4", "0.4.1", "new"));
        upsert_series(&mut registry, unreleased("fresh"));

        assert_eq!(registry.series.len(), 2);
        assert_eq!(registry.series[0].latest_patch.as_deref(), Some("0.4.1"));
        assert_eq!(registry.series[0].commit.as_deref(), Some("new"));
        assert_eq!(registry.series[1].commit.as_deref(), Some("fresh"));
    }

    // Each entry carries only what applies to it: a released series has a patch
    // version, the unreleased build has none, and nothing states in a second
    // field what `minor` already says.
    #[test]
    fn entries_carry_only_what_applies() {
        let json = serde_json::to_string(&VersionRegistry {
            series: vec![released("0.4", "0.4.1", "ccc"), unreleased("aaa")],
        })
        .unwrap();

        assert!(json.contains(r#""latestPatch":"0.4.1""#), "{json}");
        assert_eq!(json.matches("latestPatch").count(), 1, "{json}");
        assert_eq!(json.matches(UNRELEASED).count(), 2, "{json}");
    }

    // A parse failure is silent -- `run` falls back to an empty registry and
    // the next deploy writes it out, dropping every entry -- so this is the
    // check that the file on gh-pages and these structs agree.
    #[test]
    fn registry_round_trips() {
        let source = r#"{
            "series": [
                {
                    "minor": "0.4",
                    "latestPatch": "0.4.1",
                    "path": "/0.4/",
                    "commit": "346206c3a2a09435e9771329a4d6256bdcf087a4"
                },
                {
                    "minor": "unreleased",
                    "path": "/unreleased/",
                    "commit": "b6ae88327e7ffadb2371cf171b85a8b8d2b61e28"
                }
            ]
        }"#;

        let registry: VersionRegistry = serde_json::from_str(source).unwrap();
        assert_eq!(registry.series.len(), 2);
        assert!(!registry.series[0].is_unreleased());
        assert_eq!(
            registry.series[0].commit.as_deref(),
            Some("346206c3a2a09435e9771329a4d6256bdcf087a4")
        );
        assert!(registry.series[1].is_unreleased());
        assert_eq!(registry.series[1].latest_patch, None);

        let reread: VersionRegistry =
            serde_json::from_str(&serde_json::to_string(&registry).unwrap()).unwrap();
        assert_eq!(reread.series[1].commit, registry.series[1].commit);
        assert!(reread.series[1].is_unreleased());
    }
}
