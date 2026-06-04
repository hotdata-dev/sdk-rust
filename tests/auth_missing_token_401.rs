//! Scenario: auth_missing_token_401.
//!
//! Calls without a bearer token return 401 with the documented
//! ApiErrorResponse shape. Uses an unauthenticated `Configuration` built
//! locally — does not go through the ergonomic `Client` (which always installs
//! a token provider).
//!
//! Although this scenario sends no credentials, it still gates on the standard
//! test env (like sdk-python's `env` fixture) so `cargo test` with no secrets
//! configured does not make a live network call against prod.

mod common;

use hotdata::apis::workspaces_api;
use hotdata::Configuration;

#[tokio::test]
async fn auth_missing_token_401() {
    // Gate on creds so offline CI skips cleanly (mirrors python's env fixture).
    let _client = skip_if_no_creds!();
    let env = common::load_env();

    // No bearer token, no workspace header — just the API host.
    let mut config = Configuration::new();
    config.base_path = env.api_url.trim_end_matches('/').to_string();

    let result = workspaces_api::list_workspaces(&config, None).await;
    match result {
        Err(err) => {
            assert_eq!(
                common::status_of(&err),
                Some(401),
                "expected 401 without bearer token, got {err:?}"
            );
            // The documented error body is non-empty (ApiErrorResponse shape).
            if let hotdata::Error::ResponseError(content) = &err {
                assert!(
                    !content.content.is_empty(),
                    "expected non-empty error body on 401"
                );
            }
        }
        Ok(_) => panic!("list_workspaces without a token must not succeed"),
    }
}
