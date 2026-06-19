//! Scenario: indexes_lifecycle.
//!
//! Exempt for every SDK (`optional_for: [python, typescript, rust]` in
//! test-scenarios.yaml) — indexes are scoped to (connection_id, schema, table)
//! of *real* source tables. Implementing it requires a dedicated indexable test table
//! plus env vars naming the schema/table/column to target, and the generated
//! Rust client does not (yet) expose an indexes API.
//!
//! This file exists so the scenario-parity scan finds `tests/indexes_lifecycle.rs`.
//! It always skips at runtime.

mod common;

#[tokio::test]
async fn indexes_lifecycle() {
    eprintln!(
        "SKIP {}: indexes_lifecycle is exempt for all SDKs (requires a real \
         indexable source table; see test-scenarios.yaml)",
        module_path!()
    );
    // Touch the shared helper so the `common` module is considered used and the
    // file stays wired to the same gating utilities as the rest of the suite.
    let _ = common::load_env();
}
