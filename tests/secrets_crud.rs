//! Scenario: secrets_crud.
//!
//! Create, read, update, delete a secret. Critically: confirm that get_secret /
//! list_secrets never echo the plaintext value back. The typed response models
//! have no `value` field, but we also defensively serialize the payloads and
//! scan for the plaintext to catch a server-side regression where the value
//! leaks through.

mod common;

use hotdata::apis::secrets_api;
use hotdata::models;

fn assert_no_leak<T: serde::Serialize>(value: &T, secret: &str, context: &str) {
    let dumped = serde_json::to_string(value).expect("response should serialize");
    assert!(
        !dumped.contains(secret),
        "{context} response leaked plaintext secret value"
    );
}

#[tokio::test]
async fn secrets_crud() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    // Server normalizes secret names — underscores only.
    let secret_name = common::sdkci_name("secrets-crud").replace('-', "_");
    let initial_value = "INITIAL_PLAINTEXT_VALUE_DO_NOT_LEAK";
    let updated_value = "UPDATED_PLAINTEXT_VALUE_DO_NOT_LEAK";

    let create_resp = secrets_api::create_secret(
        config,
        models::CreateSecretRequest::new(secret_name.clone(), initial_value.to_string()),
    )
    .await
    .expect("create_secret should succeed");
    assert_eq!(create_resp.name, secret_name);
    assert_no_leak(&create_resp, initial_value, "create_secret");

    let got = secrets_api::get_secret(config, &secret_name)
        .await
        .expect("get_secret should succeed");
    assert_eq!(got.name, secret_name);
    assert_no_leak(&got, initial_value, "get_secret");

    let listing = secrets_api::list_secrets(config)
        .await
        .expect("list_secrets should succeed");
    assert!(
        listing.secrets.iter().any(|s| s.name == secret_name),
        "created secret {secret_name} not present in list_secrets"
    );
    assert_no_leak(&listing, initial_value, "list_secrets");

    secrets_api::update_secret(
        config,
        &secret_name,
        models::UpdateSecretRequest::new(updated_value.to_string()),
    )
    .await
    .expect("update_secret should succeed");

    let got2 = secrets_api::get_secret(config, &secret_name)
        .await
        .expect("get_secret after update should succeed");
    assert_no_leak(&got2, updated_value, "get_secret (post-update)");
    assert_no_leak(&got2, initial_value, "get_secret (post-update)");

    secrets_api::delete_secret(config, &secret_name)
        .await
        .expect("delete_secret should succeed");

    let after_delete = secrets_api::get_secret(config, &secret_name).await;
    match after_delete {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 after delete, got {err:?}"
        ),
        Ok(_) => panic!("get_secret should fail with 404 after delete"),
    }
}
