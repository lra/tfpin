# tfpin

A small, fast linter that keeps your Terraform repository on the versions and conventions you've
agreed on. Point it at a `.tfpin.toml` config and it checks every `.tf` file for:

- the expected **Terraform version** (`terraform.required_version`);
- pinned **provider versions** (`required_providers`);
- pinned **module versions** (`module { source, version }`);
- an **S3 backend convention** — fixed attributes plus a state `key` that mirrors the file's path;
- leftover **refactoring blocks** (`moved`, `import`, `removed`) that should be deleted after use.

Every rule lives in the config file, so the same binary works for any organisation. tfpin exits
`0` when everything is fine and `1` when any violation is found, which makes it a natural CI gate.

## Install

Download a binary from the [releases page](https://github.com/lra/tfpin/releases), or build from
source:

```sh
cargo install --path .
# or
cargo build --release   # -> target/release/tfpin
```

## Usage

```sh
tfpin [PATHS]...
```

- `PATHS` — directories or `.tf` files to scan. Defaults to the config file's directory. When
  given, they *narrow* (never widen) the set selected by your `include`/`exclude` globs — handy for
  checking only the files changed in a PR.
- `-c, --config <FILE>` — path to the config. By default tfpin discovers `.tfpin.toml` by walking
  up from the current directory.
- `-v, --verbose` — print each file as it is checked.

```sh
tfpin                       # check the whole repo
tfpin platform/stack/prod   # check one subtree
tfpin -c ci/.tfpin.toml -v  # explicit config, verbose
```

Exit codes: `0` clean · `1` violations found · `2` could not run (missing config, bad pattern, …).

## Configuration

Create a `.tfpin.toml` at the root of your repository. Every section is optional — a check only
runs when its section is present, and only enforces what you specify. See
[`.tfpin.example.toml`](.tfpin.example.toml) for a fully-commented template.

```toml
terraform_version = "~> 1.15.5"
forbidden_blocks  = ["moved", "import", "removed"]

[providers]
aws        = "~> 6.24"
kubernetes = "~> 2.37, >= 2.37.1"   # compound constraints compared verbatim

[modules]
"terraform-aws-modules/vpc/aws" = "~> 6.2"

[backend.s3]
allowed_account_ids = ["111111111111"]
bucket              = "acme-terraform-state"
region              = "us-west-1"
use_lockfile        = true
key_template        = "acme/infra/{dir}"
```

Notes:

- **`[backend.s3]` is free-form.** Any attribute you list is compared verbatim against the
  `backend "s3"` block (string, bool, number or array). Attributes on the block that you don't list
  are ignored, so extras like `encrypt`, `kms_key_id` or `profile` are fine.
- **`key_template`** is the one reserved key. `{dir}` is replaced by the `.tf` file's directory,
  relative to the config file, joined with `/`. For `platform/foo/main.tf` with
  `key_template = "acme/infra/{dir}"`, the expected `key` is `acme/infra/platform/foo`.
- **Skipping is intentional.** A provider/module without a `version`, or one not listed in the
  config, is left alone. The contract is "if it's pinned, it must be pinned correctly."
- `**/.terraform/**` is always excluded.

## GitHub Action

Run tfpin in CI with the bundled composite action, which downloads the matching prebuilt binary:

```yaml
name: terraform-lint
on: [pull_request]

jobs:
  tfpin:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: lra/tfpin@v0.1.0
        # with:
        #   version: v0.1.0        # or "latest" (default)
        #   config: ci/.tfpin.toml # optional
        #   args: "platform"       # extra args / paths
        #   working-directory: .
```

The job fails when tfpin finds any violation.

## Development

```sh
cargo test            # unit + integration tests (fixtures under tests/fixtures/)
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

Source layout: `src/config.rs` (the `.tfpin.toml` model), `src/hcl_nav.rs` (helpers over the
[`hcl`](https://crates.io/crates/hcl-rs) parse tree), `src/checks/` (one module per rule), and
`src/main.rs` (CLI, file discovery, output).

## License

[GPL-3.0-or-later](LICENSE).
