//! For every provider listed in the config, if a file pins that provider's `version` in
//! `required_providers`, it must match. Providers absent from the config, or declared without a
//! `version`, are left alone.

use hcl::{Body, Expression};

use super::Violation;
use crate::config::Config;
use crate::hcl_nav::{child_block, object_get, top_blocks};

pub fn check(body: &Body, cfg: &Config, out: &mut Vec<Violation>) {
    if cfg.providers.is_empty() {
        return;
    }

    for tf in top_blocks(body, "terraform") {
        let Some(required) = child_block(tf, "required_providers", None) else {
            continue;
        };

        for entry in required.body().attributes() {
            let name = entry.key();
            let Some(expected) = cfg.providers.get(name) else {
                continue;
            };
            // Only enforce when a literal `version` is actually set on the provider.
            if let Some(Expression::String(found)) = object_get(entry.expr(), "version") {
                if found != expected {
                    out.push(Violation::new(format!(
                        "wrong {name} provider version: found {found:?}, expected {expected:?}"
                    )));
                }
            }
        }
    }
}
