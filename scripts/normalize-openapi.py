#!/usr/bin/env python3
"""Prepare the OpenAPI spec for the openapi-generator Rust backend.

Two rewrites, both applied ONLY to the throwaway spec fed to the generator in
``regenerate.yml``; the canonical spec in www.hotdata.dev (and the Python/TS
SDKs) are untouched.

1. Flatten ``$ref``-with-siblings, which the Rust backend cannot name.
2. Name the anonymous result-row cell schema ``JsonCell``, so the generator can
   be told a hand-written type supplies it.

Rewrite 1 — ``$ref``-with-siblings

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
name.

Rewrite 2 — the row-cell schema

A result row is spelled ``rows: array of array of {}`` — an unnamed "any JSON
value" cell — which the Rust backend renders as ``serde_json::Value``. That
type has no arbitrary-precision number, so a decimal wider than an ``f64``
(``DECIMAL(38,2)`` at full width, say) is rounded the moment the response body
is parsed, before a caller can see it. Naming the cell schema lets
``regenerate.yml`` pass ``--import-mappings=JsonCell=crate::models::JsonCell``,
which makes the generator emit ``Vec<Vec<models::JsonCell>>`` for every
``rows`` field and skip writing the model — leaving the hand-written
``src/models/json_cell.rs``, which holds each cell's JSON text, to supply it.

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


#: Name given to the row-cell schema. Must match the ``--type-mappings`` /
#: ``--import-mappings`` pair in ``regenerate.yml`` and the hand-written
#: ``src/models/json_cell.rs`` that supplies the type.
CELL_SCHEMA = "JsonCell"


def name_row_cells(spec: object) -> list[str]:
    """Point every ``rows`` cell at a named ``JsonCell`` schema.

    Matches on shape rather than on a list of model names: a ``rows`` property
    that is an array of arrays whose element schema is empty. Returns the
    models rewritten.
    """
    if not isinstance(spec, dict):
        return []
    schemas = spec.get("components", {}).get("schemas", {})
    patched = []
    for model, schema in schemas.items():
        if not isinstance(schema, dict):
            continue
        rows = (schema.get("properties") or {}).get("rows")
        if not isinstance(rows, dict):
            continue
        row = rows.get("items")
        # rows is an array of rows; a row is an array of cells; a cell is any
        # JSON value, i.e. an empty schema.
        if not isinstance(row, dict) or row.get("type") != "array":
            continue
        cell = row.get("items")
        # Already pointed at the cell schema: a re-run over a spec this script
        # has seen. Count it as done rather than as a miss, so a second run is
        # a no-op and not a failure.
        if cell == {"$ref": f"#/components/schemas/{CELL_SCHEMA}"}:
            patched.append(model)
            continue
        # Anything else already typed is a shape this script does not know.
        if cell:
            continue
        row["items"] = {"$ref": f"#/components/schemas/{CELL_SCHEMA}"}
        patched.append(model)

    if patched:
        schemas[CELL_SCHEMA] = {
            "description": "One cell of a result row.",
            "type": ["string", "number", "boolean", "object", "array", "null"],
        }
    return patched


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: normalize-openapi.py <spec.yaml>", file=sys.stderr)
        return 2
    path = sys.argv[1]

    with open(path, encoding="utf-8") as handle:
        spec = yaml.safe_load(handle)

    removed = strip_ref_siblings(spec)

    # Fail the regen rather than emit a client that silently rounds decimals:
    # if the spec ever stops spelling result rows this way, the substitution
    # has to be re-aimed by hand, and a green regen PR would hide that.
    named = name_row_cells(spec)
    if not named:
        print(
            f"normalize-openapi: no result-row cell schema found in {path}. The spec's "
            f"`rows` shape changed; re-aim name_row_cells() and the {CELL_SCHEMA} "
            "mappings in regenerate.yml before regenerating.",
            file=sys.stderr,
        )
        return 1

    with open(path, "w", encoding="utf-8") as handle:
        yaml.safe_dump(spec, handle, sort_keys=False, allow_unicode=True)

    print(f"normalize-openapi: stripped {removed} sibling key(s) from $ref nodes in {path}")
    print(f"normalize-openapi: named the row cells of {', '.join(named)} as {CELL_SCHEMA}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
