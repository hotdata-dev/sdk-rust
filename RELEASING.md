# Releasing

Every release uses `./scripts/release.sh`. Do not bump versions, tag, or create GitHub Releases manually.

## One-time setup

- Install [GitHub CLI](https://cli.github.com/) (`gh`) and authenticate.
- Install [`cargo-edit`](https://github.com/killercup/cargo-edit) (`cargo install cargo-edit`) so `cargo set-version` is available.
- Configure the `CRATES_IO_TOKEN` secret in the `crates-io` GitHub environment that `publish.yml` uses.

## Release steps

1. Add user-facing notes under `## [Unreleased]` in `CHANGELOG.md`.
2. Prepare the release PR:

   ```bash
   ./scripts/release.sh prepare patch   # or minor | major | 1.2.3
   ```

3. Merge the PR after CI passes (including the changelog check).
4. Publish from a clean default branch checkout:

   ```bash
   git checkout main
   git pull
   ./scripts/release.sh publish
   ```

## What happens automatically

Pushing a `vX.Y.Z` tag triggers two workflows:

| Workflow | Purpose |
|----------|---------|
| `publish.yml` | `cargo publish` the crate to [crates.io](https://crates.io) (after a `cargo publish --dry-run --all-features` check) |
| `release.yml` | Create the GitHub Release with notes from `CHANGELOG.md` |

The `arrow` feature ships as part of the same crate — features are crate metadata, not a separate publish.

## Recover a missing GitHub Release

If the crates.io publish succeeded but the GitHub Release workflow failed, rerun it from `main`
without retagging:

```bash
gh workflow run "GitHub Release" --ref main -f tag=vX.Y.Z
```

The tag must already exist on the remote. The workflow checks out that tag, extracts the
matching `CHANGELOG.md` section, and creates or updates the GitHub Release.

## Enforcement

- **PR check** (`check-release.yml`): if the `[package]` version in `Cargo.toml` changes, `CHANGELOG.md` must contain a matching `## [X.Y.Z]` section.
- **Tag check** (`publish.yml`): the tag (without `v`) must match the `[package]` version in `Cargo.toml`.
- **Publish guard** (`release.sh publish`): refuses to tag if the changelog section is missing.

Together, these make it hard to ship a version without changelog notes or a GitHub Release.

## Version source

The version lives only in `Cargo.toml` `[package].version`. `release.sh` reads it with
`cargo metadata --no-deps` and bumps it with `cargo set-version`. There is no committed
`Cargo.lock` (the crate is a library), so no lockfile step is needed. Because `Cargo.toml`
is protected from the code generator via `.openapi-generator-ignore`, the regeneration
workflow owns its own patch bump with `cargo set-version --bump patch`.
