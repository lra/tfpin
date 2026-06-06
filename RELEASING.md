# Releasing tfpin

Releases are automated by [`.github/workflows/release.yml`](.github/workflows/release.yml), which
runs when you push a tag matching `v[0-9]+.*` (e.g. `v0.2.0`). One tag push produces:

1. a **GitHub Release** for the tag (`create-release`);
2. prebuilt **binaries** for four targets, attached as `tfpin-<target>.tar.gz` + `.sha256`
   (`upload-assets`) — these are what the composite [`action.yml`](action.yml) downloads;
3. a **crates.io publish** of the new version (`publish-crate`).

## Cutting a release

1. **Bump the version** in `Cargo.toml` following [semver](https://semver.org/):

   ```toml
   [package]
   version = "0.2.0"
   ```

   Run `cargo build` so `Cargo.lock` picks up the new version.

2. **Bump the version reference** in the README's GitHub Action example so users pin the new
   release:

   ```yaml
   - uses: lra/tfpin@v0.2.0
   ```

   (`@v0.2.0` resolves to the release that tag created.)

3. **Commit and push to `master`:**

   ```sh
   git add Cargo.toml Cargo.lock README.md
   git commit -m "Release 0.2.0"
   git push
   ```

4. **Tag and push the tag.** The tag must be `v` + the exact `Cargo.toml` version:

   ```sh
   git tag v0.2.0
   git push origin v0.2.0
   ```

5. **Watch the run:**

   ```sh
   gh run watch        # or: gh run list --workflow=Release
   ```

   When it's green, the GitHub Release has all four binaries and the new version is on crates.io.

## Notes & gotchas

- **Tag ↔ version must match.** The workflow does not edit `Cargo.toml`; if the tag and the
  packaged version disagree, the crates.io publish will be for whatever is in `Cargo.toml`.
- **crates.io versions are immutable.** You can never re-publish or overwrite a version, only
  publish a higher one (a mistake is corrected with `cargo yank --version X.Y.Z` plus a new
  release). The `publish-crate` step is idempotent — it checks crates.io first and **skips**
  publishing when the version already exists, so re-running a release tag will not fail.
- **Runners.** Both macOS targets build on `macos-latest`; the `x86_64-apple-darwin` binary is
  cross-compiled there (the Intel `macos-13` runners were retired).
- **Re-running a release.** If a run fails partway (e.g. a transient runner issue), you can re-run
  the failed jobs from the Actions UI or `gh run rerun <run-id>`. Because the publish step is
  idempotent and `upload-rust-binary-action` overwrites existing assets, re-runs are safe.
- **Replacing a botched tag.** If you need to move a tag to a new commit before anyone has
  consumed the release:

  ```sh
  gh release delete v0.2.0 --yes --cleanup-tag   # removes the GitHub release and remote tag
  git tag -d v0.2.0                               # remove the local tag
  # fix things, commit, push, then re-tag:
  git tag v0.2.0 && git push origin v0.2.0
  ```

  This only works for the GitHub side; a version already on crates.io stays there forever.
