//! For every module `source` listed in the config, if a file pins a `version` for a module using
//! that source, it must match. Modules whose source is not in the config are ignored; a configured
//! source declared without a `version` is skipped.

use hcl::Body;

use super::Violation;
use crate::config::Config;
use crate::hcl_nav::{attr_str, top_blocks};

pub fn check(body: &Body, cfg: &Config, out: &mut Vec<Violation>) {
    if cfg.modules.is_empty() {
        return;
    }

    for module in top_blocks(body, "module") {
        let Some(source) = attr_str(module.body(), "source") else {
            continue;
        };
        let Some(expected) = cfg.modules.get(source) else {
            continue;
        };

        let name = module.labels().first().map(|l| l.as_str()).unwrap_or("");
        if let Some(found) = attr_str(module.body(), "version") {
            if found != expected {
                out.push(Violation::new(format!(
                    "wrong {name:?} module version: found {found:?}, expected {expected:?}"
                )));
            }
        }
    }
}
