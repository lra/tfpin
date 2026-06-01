//! tfpin — lint Terraform files against pinned versions and conventions declared in `.tfpin.toml`.
//!
//! A config-driven linter for Terraform repositories. See the README and `.tfpin.toml` for the
//! configuration reference.

mod checks;
mod config;
mod hcl_nav;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use globwalk::GlobWalkerBuilder;

use config::Config;

/// Lint Terraform files against pinned versions and conventions from a `.tfpin.toml` config.
#[derive(Parser, Debug)]
#[command(name = "tfpin", version, about, long_about = None)]
struct Cli {
    /// Directories or `.tf` files to scan. Defaults to the config file's directory. When given,
    /// these restrict (never widen) the set selected by the config's `include`/`exclude` globs.
    paths: Vec<PathBuf>,

    /// Path to the config file. Defaults to discovering `.tfpin.toml` by walking up from the
    /// current directory.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Print each file as it is checked (to stderr).
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("tfpin: {message}");
            // Distinct from FAILURE (1, "violations found"): 2 means tfpin could not run.
            ExitCode::from(2)
        }
    }
}

/// Returns `Ok(true)` when any violation (or unparseable file) was found, `Ok(false)` when clean.
fn run(cli: &Cli) -> Result<bool, String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("cannot determine current directory: {e}"))?;

    let config_path = match &cli.config {
        Some(path) => path.clone(),
        None => Config::discover(&cwd).ok_or_else(|| {
            format!(
                "no {} found (searched upwards from {}); pass --config",
                config::CONFIG_FILENAME,
                cwd.display()
            )
        })?,
    };
    let config_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let cfg = Config::load(&config_path)?;

    let files = collect_files(&config_dir, &cfg, &cli.paths)?;

    let mut any_error = false;
    for file in files {
        let shown = display_path(&file, &config_dir);
        if cli.verbose {
            eprintln!("{shown}...");
        }

        let text = match std::fs::read_to_string(&file) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("tfpin: cannot read {shown}: {e}");
                any_error = true;
                continue;
            }
        };

        let body = match hcl_nav::parse(&text) {
            Ok(body) => body,
            Err(e) => {
                eprintln!("tfpin: cannot parse {shown}: {e}");
                any_error = true;
                continue;
            }
        };

        let dir = relative_dir(&file, &config_dir);
        for violation in checks::run_all(&dir, &body, &cfg) {
            println!("{shown}: {}", violation.message);
            any_error = true;
        }
    }

    Ok(any_error)
}

/// Enumerate the `.tf` files to check: the universe defined by the config's `include`/`exclude`
/// globs (relative to the config dir), optionally narrowed to the explicitly requested `paths`.
fn collect_files(
    config_dir: &Path,
    cfg: &Config,
    paths: &[PathBuf],
) -> Result<Vec<PathBuf>, String> {
    let mut patterns = cfg.include_patterns();
    for exclude in cfg.exclude_patterns() {
        // Prune the directory itself as well as its contents, so huge trees like `.terraform`
        // are never descended into.
        if let Some(dir) = exclude.strip_suffix("/**") {
            patterns.push(format!("!{dir}"));
        }
        patterns.push(format!("!{exclude}"));
    }

    let walker = GlobWalkerBuilder::from_patterns(config_dir, &patterns)
        .build()
        .map_err(|e| format!("invalid include/exclude pattern: {e}"))?;

    let mut files = Vec::new();
    for entry in walker {
        let entry = entry.map_err(|e| format!("error walking {}: {e}", config_dir.display()))?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    // Narrow to explicitly requested paths, if any.
    if !paths.is_empty() {
        let roots = canonicalize_roots(paths)?;
        files.retain(|f| {
            let canonical = f.canonicalize().unwrap_or_else(|_| f.clone());
            roots
                .iter()
                .any(|root| canonical == *root || canonical.starts_with(root))
        });
    }

    files.sort();
    files.dedup();
    Ok(files)
}

/// Canonicalize the user-supplied paths, erroring if any does not exist.
fn canonicalize_roots(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    paths
        .iter()
        .map(|p| {
            p.canonicalize()
                .map_err(|e| format!("{}: {e}", p.display()))
        })
        .collect()
}

/// The file's parent directory relative to `base`, joined with `/` (the `{dir}` placeholder value).
/// Empty when the file sits directly in `base`.
fn relative_dir(file: &Path, base: &Path) -> String {
    let relative = file.strip_prefix(base).unwrap_or(file);
    let dir = relative.parent().unwrap_or_else(|| Path::new(""));
    dir.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// A path for display: relative to the config dir when possible, otherwise as-is.
fn display_path(file: &Path, base: &Path) -> String {
    file.strip_prefix(base)
        .unwrap_or(file)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_dir_strips_base_and_uses_forward_slashes() {
        let base = Path::new("/repo");
        assert_eq!(
            relative_dir(Path::new("/repo/aws/globex/backend.tf"), base),
            "aws/globex"
        );
        assert_eq!(
            relative_dir(Path::new("/repo/platform/foo/bar/main.tf"), base),
            "platform/foo/bar"
        );
        // File directly in the config dir has an empty {dir}.
        assert_eq!(relative_dir(Path::new("/repo/main.tf"), base), "");
    }
}
