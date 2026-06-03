#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PKG_NAME="hotdata"

die() { echo "error: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "$1 is required"; }

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release.sh prepare [patch|minor|major|X.Y.Z]
  ./scripts/release.sh publish

Workflow:
  1. Move notes from [Unreleased] in CHANGELOG.md (or add them there).
  2. ./scripts/release.sh prepare patch
  3. Merge the release PR.
  4. ./scripts/release.sh publish

Tag push triggers the crates.io publish and GitHub Release creation in CI.
EOF
}

# Read the [package] version. Prefer cargo metadata (robust against dependency
# `version = ` lines); fall back to a sed anchored to the [package] table so the
# script still works when cargo is unavailable.
get_version() {
  if command -v cargo >/dev/null 2>&1 && command -v jq >/dev/null 2>&1; then
    cargo metadata --no-deps --format-version 1 \
      | jq -r --arg name "$PKG_NAME" '.packages[] | select(.name==$name) | .version'
  else
    # Fallback: read the first `version = "..."` inside the [package] table.
    awk '
      /^\[package\]/ { in_pkg = 1; next }
      /^\[/ { in_pkg = 0 }
      in_pkg && /^[[:space:]]*version[[:space:]]*=/ {
        line = $0
        sub(/^[^"]*"/, "", line)
        sub(/".*/, "", line)
        print line
        exit
      }
    ' Cargo.toml
  fi
}

set_version() {
  local ver="$1"
  if command -v cargo-set-version >/dev/null 2>&1 || cargo set-version --help >/dev/null 2>&1; then
    cargo set-version "$ver"
  else
    # Fallback: edit only the first `version = "..."` line inside [package].
    local tmp
    tmp="$(mktemp)"
    awk -v ver="$ver" '
      /^\[package\]/ { in_pkg = 1 }
      /^\[/ && $0 != "[package]" { in_pkg = 0 }
      {
        if (in_pkg && !done && $0 ~ /^[[:space:]]*version[[:space:]]*=/) {
          sub(/=[[:space:]]*"[^"]*"/, "= \"" ver "\"")
          done = 1
        }
        print
      }
    ' Cargo.toml > "$tmp"
    mv "$tmp" Cargo.toml
  fi
}

bump_version() {
  local kind="$1" current="$2"
  python3 - "$kind" "$current" <<'PY'
import re, sys
kind, current = sys.argv[1], sys.argv[2]
match = re.match(r"^(\d+)\.(\d+)\.(\d+)(.*)$", current)
if not match:
    raise SystemExit(f"unsupported version: {current}")
major, minor, patch, suffix = int(match[1]), int(match[2]), int(match[3]), match[4]
if suffix:
    raise SystemExit("pre-release versions must be set explicitly as X.Y.Z")
if kind == "patch":
    patch += 1
elif kind == "minor":
    minor += 1
    patch = 0
elif kind == "major":
    major += 1
    minor = 0
    patch = 0
else:
    raise SystemExit(f"unknown bump kind: {kind}")
print(f"{major}.{minor}.{patch}")
PY
}

default_branch() {
  local remote="${1:-origin}"
  git symbolic-ref --quiet "refs/remotes/${remote}/HEAD" 2>/dev/null | sed "s|refs/remotes/${remote}/||" \
    || { git branch -r | sed -n "s|^  ${remote}/\\(main\\|master\\)$|\\1|p" | head -1; } \
    || echo main
}

ensure_clean() {
  [[ -z "$(git status --porcelain)" ]] || die "working tree is not clean"
}

update_changelog() {
  local ver="$1"
  local date
  date="$(date +%Y-%m-%d)"
  python3 scripts/update_changelog.py "$ver" "$date"
}

cmd_prepare() {
  local bump="${1:-}"
  [[ -n "$bump" ]] || { usage; die "missing bump kind or explicit version"; }
  need gh
  need python3
  need git
  ensure_clean

  local current new base branch
  current="$(get_version)"
  [[ -n "$current" ]] || die "could not read current version from Cargo.toml"
  if [[ "$bump" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    new="$bump"
  else
    new="$(bump_version "$bump" "$current")"
  fi
  [[ "$new" != "$current" ]] || die "new version ($new) equals current ($current)"

  base="$(default_branch)"
  git fetch origin "$base"
  git checkout "$base"
  git pull --ff-only origin "$base"
  ensure_clean

  set_version "$new"
  update_changelog "$new"

  branch="release/v${new}"
  git checkout -b "$branch"
  git add Cargo.toml CHANGELOG.md
  git commit -m "chore: release v${new}"

  git push -u origin "$branch"
  gh pr create --base "$base" --head "$branch" \
    --title "chore: release ${PKG_NAME} v${new}" \
    --body "Release **${PKG_NAME} v${new}**: version bumped in \`Cargo.toml\` and \`CHANGELOG.md\` updated. After merge, run \`./scripts/release.sh publish\` from a clean \`${base}\` checkout."

  echo "Prepared ${PKG_NAME} v${new}. Merge the PR, then run: ./scripts/release.sh publish"
}

cmd_publish() {
  need gh
  need python3
  need git
  ensure_clean

  local base ver tag
  base="$(default_branch)"
  git fetch origin "$base"
  git checkout "$base"
  git pull --ff-only origin "$base"
  ensure_clean

  ver="$(get_version)"
  [[ -n "$ver" ]] || die "could not read version from Cargo.toml"
  tag="v${ver}"

  git rev-parse "$tag" >/dev/null 2>&1 && die "tag $tag already exists"
  [[ -f CHANGELOG.md ]] || die "CHANGELOG.md is required"
  python3 - "$ver" <<'PY'
import re, sys
from pathlib import Path
ver = sys.argv[1]
text = Path("CHANGELOG.md").read_text()
if not re.search(rf"^## \[{re.escape(ver)}\]", text, re.M):
    raise SystemExit(f"CHANGELOG.md missing section for {ver}")
PY

  git tag "$tag"
  git push origin "$tag"

  echo "Pushed ${tag} for ${PKG_NAME}."
  echo "CI will publish to crates.io and create the GitHub Release."
}

case "${1:-}" in
  prepare) shift; cmd_prepare "${1:-}" ;;
  publish) cmd_publish ;;
  -h|--help|help|"") usage ;;
  *) usage; die "unknown command: $1" ;;
esac
