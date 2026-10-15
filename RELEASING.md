# Releasing

Every release uses `./scripts/release.sh`. Do not bump versions, tag, or create GitHub Releases manually.

## One-time setup

- Install [GitHub CLI](https://cli.github.com/) (`gh`) and authenticate.
- Configure the `CRATES_IO_TOKEN` secret in the `crates-io` GitHub environment that `publish.yml` uses.
- `cut-release.yml` needs the `HOTDATA_AUTOMATION_PRIVATE_KEY` secret (the same GitHub App the
  regeneration workflow uses). The App token is what lets the release PR get CI checks and the
  tag trigger publishing: anything opened or pushed with `GITHUB_TOKEN` triggers no workflows.

## Release steps

1. Make sure `## [Unreleased]` in `CHANGELOG.md` has user-facing notes.
2. Cut the release from CI:

   ```bash
   gh workflow run cut-release.yml -f bump=minor   # or patch | major | 1.2.3
   ```

   This runs `./scripts/release.sh prepare` on a runner: it bumps `Cargo.toml`, rolls
   `[Unreleased]` into a `## [X.Y.Z]` section, opens the `release/vX.Y.Z` PR, and enables
   auto-merge on it.
3. The PR merges on its own once CI (including the changelog check) and the review gate pass.
4. Merging the PR runs `./scripts/release.sh publish` in CI, which tags `main` with `vX.Y.Z`.
   The tag push publishes the crate and creates the GitHub Release.

## What happens automatically

Pushing a `vX.Y.Z` tag triggers two workflows:

| Workflow | Purpose |
|----------|---------|
| `publish.yml` | `cargo publish` the crate to [crates.io](https://crates.io) (after a `cargo publish --dry-run --all-features` check) |
| `release.yml` | Create the GitHub Release with notes from `CHANGELOG.md` |

The `arrow` feature ships as part of the same crate — features are crate metadata, not a separate publish.

## Recover a missing tag

If the release PR merged but the `Tag release` job failed (so no `vX.Y.Z` tag exists), rerun
the failed job from the Actions tab, or tag from a clean local checkout:

```bash
git checkout main
git pull
./scripts/release.sh publish
```

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
- **Publish guard** (`release.sh publish`): refuses to tag if the changelog section is missing or the tag already exists.

Together, these make it hard to ship a version without changelog notes or a GitHub Release.

## Version source

The version lives only in `Cargo.toml` `[package].version`. `release.sh` reads it with
`cargo metadata --no-deps` and bumps it with `cargo set-version`. There is no committed
`Cargo.lock` (the crate is a library), so no lockfile step is needed. Because `Cargo.toml`
is protected from the code generator via `.openapi-generator-ignore`, the regeneration
workflow owns its own patch bump with `cargo set-version --bump patch`.
