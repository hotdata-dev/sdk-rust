//! Scenario: default_user_agent.
//!
//! The generated `Configuration::default()` user-agent must always reflect the
//! crate version that is actually compiled/shipped, not a version baked in at
//! the last regen. The template computes it from `CARGO_PKG_VERSION` at compile
//! time (the same mechanism `ClientBuilder` uses), so the default UA can never
//! lag behind the published version. Pure local assertion — no network/creds.

use hotdata::Configuration;

#[test]
fn default_user_agent_matches_crate_version() {
    let expected = concat!("hotdata-rust/", env!("CARGO_PKG_VERSION"));
    assert_eq!(
        Configuration::default().user_agent.as_deref(),
        Some(expected),
        "default user-agent must match the compiled crate version"
    );
}
