#!/usr/bin/env python3
"""Flatten ``$ref``-with-siblings so the openapi-generator Rust backend can run.

OpenAPI 3.1 (JSON Schema 2020-12) lets a ``$ref`` carry sibling keywords — e.g. a
per-branch ``description`` on a ``oneOf`` member. That is valid, and the Python
and TypeScript generators consume it fine, but the Rust generator NPEs in
``AbstractRustCodegen.toModelName`` because it materializes the branch as an
*unnamed* inline schema. (Reproduced on generator 7.20.0 and 7.22.0; the upstream
backend has no fix yet.) The fatal case in our spec is ``JobResult``'s ``oneOf``,
whose four ``$ref`` branches each carry a ``description``.

We don't need those siblings in the generated Rust client — per-branch ``oneOf``
docs don't render in Rust regardless — so for every node that has a ``$ref``
alongside other keys we drop the other keys, leaving a pure ref the generator can
name. This rewrites ONLY the throwaway spec fed to the generator in
``regenerate.yml``; the canonical spec in www.hotdata.dev (and the Python/TS
SDKs) are untouched.

Usage: ``normalize-openapi.py <spec.yaml>`` (edits the file in place).
"""

from __future__ import annotations

import sys

import yaml


def strip_ref_siblings(node: object) -> int:
    """Recursively drop keys that sit alongside a ``$ref``. Returns the count."""
    removed = 0
    if isinstance(node, dict):
        if "$ref" in node and len(node) > 1:
            for key in [k for k in node if k != "$ref"]:
                del node[key]
                removed += 1
        for value in list(node.values()):
            removed += strip_ref_siblings(value)
    elif isinstance(node, list):
        for item in node:
            removed += strip_ref_siblings(item)
    return removed


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: normalize-openapi.py <spec.yaml>", file=sys.stderr)
        return 2
    path = sys.argv[1]

    with open(path, encoding="utf-8") as handle:
        spec = yaml.safe_load(handle)

    removed = strip_ref_siblings(spec)

    with open(path, "w", encoding="utf-8") as handle:
        yaml.safe_dump(spec, handle, sort_keys=False, allow_unicode=True)

    print(f"normalize-openapi: stripped {removed} sibling key(s) from $ref nodes in {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
