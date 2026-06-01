//! When a file declares a `backend "s3"` block, enforce the configured backend convention.
//!
//! Two kinds of rule:
//!   * every attribute listed under `[backend.s3]` must be present on the block with the configured
//!     value (free-form: any string/bool/number/array is compared verbatim); and
//!   * `key_template` is rendered with `{dir}` replaced by the file's directory (relative to the
//!     config file) and compared against the block's `key`.
//!
//! Files without an `s3` backend are skipped entirely.

use hcl::Body;

use super::Violation;
use crate::config::Config;
use crate::hcl_nav::{
    attr, attr_str, child_block, expr_display, expr_matches, toml_display, top_blocks,
};

pub fn check(dir: &str, body: &Body, cfg: &Config, out: &mut Vec<Violation>) {
    let Some(s3) = cfg.backend.s3.as_ref() else {
        return;
    };

    for tf in top_blocks(body, "terraform") {
        let Some(backend) = child_block(tf, "backend", Some("s3")) else {
            continue;
        };
        let bbody = backend.body();

        // Free-form expected attributes.
        for (name, expected) in &s3.expected {
            match attr(bbody, name) {
                None => out.push(Violation::new(format!(
                    "S3 backend missing {name} (expected {})",
                    toml_display(expected)
                ))),
                Some(found) if !expr_matches(found, expected) => out.push(Violation::new(format!(
                    "S3 backend {name} is {} (expected {})",
                    expr_display(found),
                    toml_display(expected)
                ))),
                Some(_) => {}
            }
        }

        // Path-mirroring `key`.
        if let Some(template) = &s3.key_template {
            let expected_key = template.replace("{dir}", dir);
            match attr_str(bbody, "key") {
                None => out.push(Violation::new(format!(
                    "S3 backend missing key (expected {expected_key:?})"
                ))),
                Some(found) if found != expected_key => out.push(Violation::new(format!(
                    "S3 backend key {found:?} should match the workspace path: {expected_key:?}"
                ))),
                Some(_) => {}
            }
        }
    }
}
