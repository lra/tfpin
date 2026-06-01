//! End-to-end tests driving the built `tfpin` binary against fixture trees under
//! `tests/fixtures/`. The fixtures cover realistic Terraform layouts and edge cases.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A `tfpin` command run with its working directory set to the named fixture, so config discovery
/// finds that fixture's `.tfpin.toml`.
fn tfpin_in(fixture_name: &str) -> Command {
    let mut cmd = Command::cargo_bin("tfpin").unwrap();
    cmd.current_dir(fixture(fixture_name));
    cmd
}

// --- Working scenarios: clean trees pass with no output. -----------------------------------------

#[test]
fn acme_good_tree_passes() {
    tfpin_in("acme_good")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn globex_good_tree_passes() {
    // Same engine, a different organisation's convention (trailing-filename key, extra S3 attrs).
    tfpin_in("globex_good")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn terraform_cache_files_are_never_scanned() {
    // acme_good contains a deliberately-wrong file under .terraform/. The tree only passes if
    // it is excluded by default.
    let junk = fixture("acme_good").join("platform/cloud/confluent/.terraform/modules/junk.tf");
    assert!(
        junk.exists(),
        "the .terraform fixture file must exist for this test to mean anything"
    );
    tfpin_in("acme_good").assert().success();
}

// --- Parser robustness: tricky HCL must parse and behave, never panic. ---------------------------

#[test]
fn tricky_hcl_parses_and_passes() {
    // Heredocs, configuration_aliases with trailing comma, for_each, comments, var traversals.
    tfpin_in("parse")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn malformed_file_is_reported_not_panicked() {
    tfpin_in("parse_error")
        .assert()
        .failure()
        .stderr(contains("cannot parse").and(contains("broken.tf")));
}

// --- Failure scenarios: each violation type is detected. -----------------------------------------

#[test]
fn acme_bad_tree_fails_with_expected_violations() {
    tfpin_in("acme_bad")
        .assert()
        .failure()
        .stdout(contains(
            r#"wrong terraform required_version: found "~> 1.14.0", expected "~> 1.15.5""#,
        ))
        .stdout(contains(
            r#"wrong aws provider version: found "~> 5.0", expected "~> 6.24""#,
        ))
        .stdout(contains("wrong kubernetes provider version"))
        .stdout(contains(
            r#"wrong "vpc" module version: found "~> 5.0", expected "~> 6.2""#,
        ))
        .stdout(contains(
            "S3 backend region is us-east-1 (expected us-west-1)",
        ))
        .stdout(contains(
            "S3 backend bucket is some-other-bucket (expected acme-terraform-state)",
        ))
        .stdout(contains("S3 backend use_lockfile is false (expected true)"))
        .stdout(contains(
            "S3 backend allowed_account_ids is [000000000000] (expected [111111111111])",
        ))
        .stdout(contains("should match the workspace path"))
        .stdout(contains(r#"found refactoring block "moved" leftover"#))
        .stdout(contains(r#"found refactoring block "import" leftover"#))
        .stdout(contains(r#"found refactoring block "removed" leftover"#));
}

// --- Path narrowing and config/discovery behaviour. ----------------------------------------------

#[test]
fn explicit_path_narrows_the_scan() {
    // Restricting to the module directory yields only the module violation, none of the version
    // or backend violations elsewhere in the bad tree.
    tfpin_in("acme_bad")
        .arg("platform/stack/badmod")
        .assert()
        .failure()
        .stdout(contains(r#"wrong "vpc" module version"#))
        .stdout(contains("required_version").not())
        .stdout(contains("S3 backend").not());
}

#[test]
fn explicit_config_flag_is_honoured() {
    // Run from the repo root, pointing at the good config explicitly, scanning its tree.
    Command::cargo_bin("tfpin")
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("--config")
        .arg(fixture("acme_good").join(".tfpin.toml"))
        .arg(fixture("acme_good"))
        .assert()
        .success();
}

#[test]
fn missing_config_is_an_operational_error() {
    let empty = tempfile::tempdir().unwrap();
    Command::cargo_bin("tfpin")
        .unwrap()
        .current_dir(empty.path())
        .assert()
        .code(2)
        .stderr(contains("no .tfpin.toml found"));
}
