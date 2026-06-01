//! Refactoring blocks (`moved`, `import`, `removed`) are only useful until the refactor they
//! describe has been applied, so any left behind is flagged.

use hcl::Body;

use super::Violation;
use crate::config::Config;
use crate::hcl_nav::top_blocks;

pub fn check(body: &Body, cfg: &Config, out: &mut Vec<Violation>) {
    for kind in &cfg.forbidden_blocks {
        if top_blocks(body, kind).next().is_some() {
            out.push(Violation::new(format!(
                "found refactoring block {kind:?} leftover (should be removed after use)"
            )));
        }
    }
}
