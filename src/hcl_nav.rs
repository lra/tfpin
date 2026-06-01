//! Thin navigation helpers over the `hcl` crate's parse tree.
//!
//! Keeping all of the `hcl`-specific traversal here lets the individual checks read as simple
//! lookups (terraform -> backend -> s3) instead of being littered with pattern matches.

use hcl::{Block, Body, Expression, ObjectKey};

/// Parse Terraform source into a [`Body`], returning a readable error on failure.
pub fn parse(text: &str) -> Result<Body, String> {
    hcl::parse(text).map_err(|e| e.to_string())
}

/// Iterate top-level blocks of `body` whose identifier matches `ident` (e.g. `terraform`).
pub fn top_blocks<'a>(body: &'a Body, ident: &str) -> impl Iterator<Item = &'a Block> {
    let ident = ident.to_string();
    body.blocks().filter(move |b| b.identifier() == ident)
}

/// Find the first child block of `parent` with the given `ident` and, when `label` is `Some`, a
/// matching first label. Used for `required_providers` (no label) and `backend "s3"` (label).
pub fn child_block<'a>(parent: &'a Block, ident: &str, label: Option<&str>) -> Option<&'a Block> {
    parent.body().blocks().find(|b| {
        b.identifier() == ident
            && match label {
                None => true,
                Some(want) => b.labels().first().map(|l| l.as_str()) == Some(want),
            }
    })
}

/// Read an attribute expression by key from a block body.
pub fn attr<'a>(body: &'a Body, key: &str) -> Option<&'a Expression> {
    body.attributes().find(|a| a.key() == key).map(|a| a.expr())
}

/// Read an attribute as a plain string, if it is a literal string.
pub fn attr_str<'a>(body: &'a Body, key: &str) -> Option<&'a str> {
    match attr(body, key)? {
        Expression::String(s) => Some(s),
        _ => None,
    }
}

/// Look up a key inside an object expression (e.g. the `version` of a `required_providers` entry).
pub fn object_get<'a>(expr: &'a Expression, key: &str) -> Option<&'a Expression> {
    let Expression::Object(obj) = expr else {
        return None;
    };
    obj.iter().find_map(|(k, v)| match k {
        ObjectKey::Identifier(id) if id.as_str() == key => Some(v),
        ObjectKey::Expression(Expression::String(s)) if s == key => Some(v),
        _ => None,
    })
}

/// Convert a literal HCL expression into the equivalent [`toml::Value`] so it can be compared
/// directly against a configured expected value. Returns `None` for non-literal expressions
/// (variables, traversals, function calls, interpolated templates, …).
pub fn expr_to_toml(expr: &Expression) -> Option<toml::Value> {
    match expr {
        Expression::String(s) => Some(toml::Value::String(s.clone())),
        Expression::Bool(b) => Some(toml::Value::Boolean(*b)),
        Expression::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(toml::Value::Integer(i))
            } else {
                n.as_f64().map(toml::Value::Float)
            }
        }
        Expression::Array(items) => items
            .iter()
            .map(expr_to_toml)
            .collect::<Option<Vec<_>>>()
            .map(toml::Value::Array),
        _ => None,
    }
}

/// Compare a parsed expression against an expected config value.
pub fn expr_matches(expr: &Expression, expected: &toml::Value) -> bool {
    expr_to_toml(expr).as_ref() == Some(expected)
}

/// Render an expression for use in a human-readable violation message.
pub fn expr_display(expr: &Expression) -> String {
    match expr {
        Expression::String(s) => s.clone(),
        Expression::Bool(b) => b.to_string(),
        Expression::Number(n) => n.to_string(),
        Expression::Array(items) => {
            let inner: Vec<String> = items.iter().map(expr_display).collect();
            format!("[{}]", inner.join(", "))
        }
        Expression::Null => "null".to_string(),
        _ => "<expression>".to_string(),
    }
}

/// Render a configured expected [`toml::Value`] in the same style as [`expr_display`], so
/// found-vs-expected messages line up.
pub fn toml_display(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(toml_display).collect();
            format!("[{}]", inner.join(", "))
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
terraform {
  required_version = "~> 1.15.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.24"
    }
    google = {
      source = "hashicorp/google"
    }
  }
  backend "s3" {
    bucket              = "my-bucket"
    use_lockfile        = true
    allowed_account_ids = ["111111111111"]
    key                 = "acme/infra/platform/foo"
  }
}

module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 6.2"
}
"#;

    #[test]
    fn navigates_nested_blocks_and_objects() {
        let body = parse(SAMPLE).unwrap();

        let tf = top_blocks(&body, "terraform").next().unwrap();
        assert_eq!(attr_str(tf.body(), "required_version"), Some("~> 1.15.5"));

        let rp = child_block(tf, "required_providers", None).unwrap();
        let aws = attr(rp.body(), "aws").unwrap();
        assert_eq!(
            object_get(aws, "version").and_then(|v| match v {
                Expression::String(s) => Some(s.as_str()),
                _ => None,
            }),
            Some("~> 6.24")
        );
        // A provider declared without a version yields None (must be skipped, not flagged).
        let google = attr(rp.body(), "google").unwrap();
        assert!(object_get(google, "version").is_none());

        let backend = child_block(tf, "backend", Some("s3")).unwrap();
        assert_eq!(
            attr_str(backend.body(), "key"),
            Some("acme/infra/platform/foo")
        );
    }

    #[test]
    fn compares_literals_against_toml() {
        let body = parse(SAMPLE).unwrap();
        let backend = child_block(
            top_blocks(&body, "terraform").next().unwrap(),
            "backend",
            Some("s3"),
        )
        .unwrap();

        assert!(expr_matches(
            attr(backend.body(), "use_lockfile").unwrap(),
            &toml::Value::from(true)
        ));
        assert!(expr_matches(
            attr(backend.body(), "bucket").unwrap(),
            &toml::Value::from("my-bucket")
        ));
        assert!(expr_matches(
            attr(backend.body(), "allowed_account_ids").unwrap(),
            &toml::Value::try_from(vec!["111111111111"]).unwrap()
        ));
        // Mismatch.
        assert!(!expr_matches(
            attr(backend.body(), "use_lockfile").unwrap(),
            &toml::Value::from(false)
        ));
    }
}
