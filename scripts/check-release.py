#!/usr/bin/env python3
"""Fail CI when the Cargo.toml package version changes without a matching CHANGELOG entry."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path


def git_show(path: str, ref: str) -> str:
    try:
        return subprocess.check_output(["git", "show", f"{ref}:{path}"], text=True)
    except subprocess.CalledProcessError:
        return ""


def read_version(text: str) -> str:
    """Read the package version from a Cargo.toml's [package] section only.

    Anchors to the [package] table so dependency `version = "..."` lines never
    match, mirroring `cargo metadata` semantics without needing a toolchain.
    """
    in_package = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_package = stripped == "[package]"
            continue
        if in_package:
            match = re.match(r'\s*version\s*=\s*"([^"]+)"', line)
            if match:
                return match.group(1)
    raise SystemExit("could not read version from Cargo.toml [package] section")


def has_changelog_section(version: str) -> bool:
    changelog = Path("CHANGELOG.md")
    if not changelog.exists():
        return False
    return bool(re.search(rf"^## \[{re.escape(version)}\]", changelog.read_text(), re.M))


def main() -> None:
    base = "origin/main"
    for candidate in ("origin/main", "origin/master"):
        if subprocess.call(
            ["git", "rev-parse", "--verify", candidate],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ) == 0:
            base = candidate
            break

    current = Path("Cargo.toml").read_text()
    previous = git_show("Cargo.toml", base)
    if not previous:
        print("skip: no base Cargo.toml to compare")
        return

    old_version = read_version(previous)
    new_version = read_version(current)
    if old_version == new_version:
        print(f"version unchanged ({new_version})")
        return

    if not has_changelog_section(new_version):
        raise SystemExit(
            f"Cargo.toml version bumped to {new_version} but CHANGELOG.md "
            f"has no '## [{new_version}]' section"
        )

    print(f"release metadata ok for {new_version}")


if __name__ == "__main__":
    main()
