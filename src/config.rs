//! Configuration model for tfpin.
//!
//! All rules are declared in a `.tfpin.toml` file. Every section is optional: a check is only
//! performed when its corresponding section is present, so an empty config enforces nothing.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Name of the configuration file discovered by walking up the directory tree.
pub const CONFIG_FILENAME: &str = ".tfpin.toml";

/// Default glob used when `include` is omitted: every Terraform file, recursively.
const DEFAULT_INCLUDE: &str = "**/*.tf";

/// Glob that is always excluded, even when the user supplies their own `exclude` list. Terraform's
/// provider cache lives under `.terraform/` and can contain thousands of vendored `.tf` files that
/// must never be linted.
const DEFAULT_EXCLUDE: &str = "**/.terraform/**";

/// The full set of rules to enforce, deserialized from `.tfpin.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Expected value of `terraform.required_version` when a file sets it.
    #[serde(default)]
    pub terraform_version: Option<String>,

    /// Top-level block kinds that must not be present (e.g. `moved`, `import`, `removed`).
    #[serde(default)]
    pub forbidden_blocks: Vec<String>,

    /// Glob patterns (relative to the config file's directory) selecting files to check.
    #[serde(default)]
    pub include: Vec<String>,

    /// Glob patterns to exclude. `**/.terraform/**` is always excluded in addition to these.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Expected provider version constraints, keyed by provider name.
    #[serde(default)]
    pub providers: BTreeMap<String, String>,

    /// Expected module version constraints, keyed by module `source`.
    #[serde(default)]
    pub modules: BTreeMap<String, String>,

    /// Backend conventions.
    #[serde(default)]
    pub backend: Backend,
}

/// Backend conventions. Only S3 is supported in v1.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backend {
    pub s3: Option<BackendS3>,
}

/// Expected attributes of an `s3` backend block.
///
/// This is intentionally free-form: any attribute the user lists is enforced verbatim against the
/// parsed backend block, and attributes present on the block but absent here are ignored. Real
/// backends carry many attributes (`encrypt`, `kms_key_id`, `profile`, …), so a fixed struct would
/// be too rigid.
///
/// `key_template` is the one reserved key: instead of a literal `key` comparison it drives the
/// path-mirroring check (see [`crate::checks::backend`]).
#[derive(Debug, Deserialize)]
pub struct BackendS3 {
    /// Template for the backend `key`, where `{dir}` is replaced by the `.tf` file's directory
    /// relative to the config file, joined with `/`.
    pub key_template: Option<String>,

    /// Every other attribute is an expected literal value (string / bool / number / string array).
    #[serde(flatten)]
    pub expected: BTreeMap<String, toml::Value>,
}

impl Config {
    /// Load and parse a config file at `path`.
    pub fn load(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read config {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("invalid config {}: {e}", path.display()))
    }

    /// Walk up from `start` looking for a `.tfpin.toml`. Returns its path when found.
    pub fn discover(start: &Path) -> Option<PathBuf> {
        for dir in start.ancestors() {
            let candidate = dir.join(CONFIG_FILENAME);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    /// Include globs, falling back to the default when none are configured.
    pub fn include_patterns(&self) -> Vec<String> {
        if self.include.is_empty() {
            vec![DEFAULT_INCLUDE.to_string()]
        } else {
            self.include.clone()
        }
    }

    /// Exclude globs, always containing the default `.terraform` exclusion.
    pub fn exclude_patterns(&self) -> Vec<String> {
        let mut patterns = self.exclude.clone();
        if !patterns.iter().any(|p| p == DEFAULT_EXCLUDE) {
            patterns.push(DEFAULT_EXCLUDE.to_string());
        }
        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let cfg: Config = toml::from_str(
            r#"
            terraform_version = "~> 1.15.5"
            forbidden_blocks  = ["moved", "import", "removed"]
            include = ["platform/**/*.tf"]

            [providers]
            aws = "~> 6.24"

            [modules]
            "terraform-aws-modules/vpc/aws" = "~> 6.2"

            [backend.s3]
            bucket       = "my-bucket"
            region       = "us-west-1"
            use_lockfile = true
            allowed_account_ids = ["111111111111"]
            key_template = "acme/infra/{dir}"
            "#,
        )
        .unwrap();

        assert_eq!(cfg.terraform_version.as_deref(), Some("~> 1.15.5"));
        assert_eq!(cfg.forbidden_blocks, ["moved", "import", "removed"]);
        assert_eq!(cfg.providers["aws"], "~> 6.24");
        assert_eq!(cfg.modules["terraform-aws-modules/vpc/aws"], "~> 6.2");

        let s3 = cfg.backend.s3.as_ref().unwrap();
        assert_eq!(s3.key_template.as_deref(), Some("acme/infra/{dir}"));
        // key_template must NOT leak into the free-form expected map.
        assert!(!s3.expected.contains_key("key_template"));
        assert_eq!(s3.expected["region"], toml::Value::from("us-west-1"));
        assert_eq!(s3.expected["use_lockfile"], toml::Value::from(true));
    }

    #[test]
    fn empty_config_is_valid_and_checks_nothing() {
        let cfg: Config = toml::from_str("").unwrap();
        assert!(cfg.terraform_version.is_none());
        assert!(cfg.forbidden_blocks.is_empty());
        assert!(cfg.providers.is_empty());
        assert!(cfg.backend.s3.is_none());
    }

    #[test]
    fn include_defaults_to_all_tf_files() {
        let cfg = Config::default();
        assert_eq!(cfg.include_patterns(), ["**/*.tf"]);
    }

    #[test]
    fn terraform_cache_is_always_excluded() {
        let cfg = Config::default();
        assert!(
            cfg.exclude_patterns()
                .contains(&"**/.terraform/**".to_string())
        );

        let cfg = Config {
            exclude: vec!["custom/**".to_string()],
            ..Config::default()
        };
        let patterns = cfg.exclude_patterns();
        assert!(patterns.contains(&"custom/**".to_string()));
        assert!(patterns.contains(&"**/.terraform/**".to_string()));
    }

    #[test]
    fn unknown_top_level_key_is_rejected() {
        let err = toml::from_str::<Config>("typo_version = \"1.0\"").unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }
}
