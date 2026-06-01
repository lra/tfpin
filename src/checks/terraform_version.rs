//! If a file sets `terraform.required_version`, it must match the configured value. Setting it is
//! not required, but when set it must be correct.

use hcl::Body;

use super::Violation;
use crate::config::Config;
use crate::hcl_nav::{attr_str, top_blocks};

pub fn check(body: &Body, cfg: &Config, out: &mut Vec<Violation>) {
    let Some(expected) = &cfg.terraform_version else {
        return;
    };

    for tf in top_blocks(body, "terraform") {
        if let Some(found) = attr_str(tf.body(), "required_version") {
            if found != expected {
                out.push(Violation::new(format!(
                    "wrong terraform required_version: found {found:?}, expected {expected:?}"
                )));
            }
        }
    }
}
