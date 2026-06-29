//! Scenario: default_user_agent.
//!
//! The generated `Configuration::default()` user-agent must always reflect the
//! crate version that is actually compiled/shipped, not a version baked in at
//! the last regen. The template computes it from `CARGO_PKG_VERSION` at compile
//! time (the same mechanism `ClientBuilder` uses), so the default UA can never
//! lag behind the published version. Pure local assertion — no network/creds.
//!
//! The expected version is read independently from Cargo.toml (the source of
//! truth) rather than from `CARGO_PKG_VERSION`: asserting against the same
//! `env!` the source uses would be circular and would not catch a regen that
//! re-bakes a stale literal into the generated `configuration.rs`.

use hotdata::Configuration;

/// Read `[package] version` from the crate's Cargo.toml without pulling in a
/// TOML dependency. Scans the `[package]` table for the first `version = "..."`.
fn cargo_toml_package_version() -> String {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("Cargo.toml should be readable");

    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = trimmed.strip_prefix("version") {
                let rest = rest.trim_start();
                if let Some(rest) = rest.strip_prefix('=') {
                    return rest.trim().trim_matches('"').to_owned();
                }
            }
        }
    }
    panic!("could not find [package] version in Cargo.toml");
}

#[test]
fn default_user_agent_matches_crate_version() {
    let expected = format!("hotdata-rust/{}", cargo_toml_package_version());
    assert_eq!(
        Configuration::default().user_agent.as_deref(),
        Some(expected.as_str()),
        "default user-agent must match the [package] version in Cargo.toml"
    );
}
