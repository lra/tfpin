//! The individual lint checks, one module per rule.
//!
//! Every check is gated on its configuration section being present and only enforces what the
//! config actually specifies, so an empty `.tfpin.toml` reports nothing.

mod backend;
mod forbidden_blocks;
mod modules;
mod providers;
mod terraform_version;

use hcl::Body;

use crate::config::Config;

/// A single rule violation found in a file. The offending file is prefixed by the caller, so the
/// message itself only describes *what* is wrong.
#[derive(Debug)]
pub struct Violation {
    pub message: String,
}

impl Violation {
    fn new(message: impl Into<String>) -> Violation {
        Violation {
            message: message.into(),
        }
    }
}

/// Run every check against one parsed file.
///
/// `dir` is the file's parent directory expressed relative to the config file, '/'-joined; it is
/// substituted into the backend `key_template`'s `{dir}` placeholder.
///
/// When `ignore_forbidden_blocks` is set, the `forbidden_blocks` check is skipped entirely
/// regardless of what the config declares.
pub fn run_all(
    dir: &str,
    body: &Body,
    cfg: &Config,
    ignore_forbidden_blocks: bool,
) -> Vec<Violation> {
    let mut out = Vec::new();
    terraform_version::check(body, cfg, &mut out);
    providers::check(body, cfg, &mut out);
    modules::check(body, cfg, &mut out);
    backend::check(dir, body, cfg, &mut out);
    if !ignore_forbidden_blocks {
        forbidden_blocks::check(body, cfg, &mut out);
    }
    out
}
