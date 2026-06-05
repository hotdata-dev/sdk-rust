//! Scenario: embedding_providers_crud.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — register a `local`
//! embedding provider (no external credentials), read it, confirm it appears in
//! list_embedding_providers, update it, then delete it. `provider_type=local`
//! keeps the test from needing a real OpenAI key.

mod common;

use hotdata::apis::embedding_providers_api;
use hotdata::models;

#[tokio::test]
async fn embedding_providers_crud() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    // Provider names follow the same underscore-normalized convention as secrets.
    let name = common::sdkci_name("embprov-crud").replace('-', "_");
    let renamed = format!("{name}_renamed");

    let created = embedding_providers_api::create_embedding_provider(
        config,
        models::CreateEmbeddingProviderRequest::new(name.clone(), "local".to_string()),
    )
    .await
    .expect("create_embedding_provider should succeed");
    assert_eq!(created.name, name);
    assert_eq!(created.provider_type, "local");
    assert!(!created.id.is_empty(), "created provider must have an id");

    let fetched = embedding_providers_api::get_embedding_provider(config, &created.id)
        .await
        .expect("get_embedding_provider should succeed");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, name);
    assert_eq!(fetched.provider_type, "local");

    let listing = embedding_providers_api::list_embedding_providers(config)
        .await
        .expect("list_embedding_providers should succeed");
    assert!(
        listing
            .embedding_providers
            .iter()
            .any(|p| p.id == created.id),
        "created provider {} not present in list_embedding_providers",
        created.id
    );

    let mut update = models::UpdateEmbeddingProviderRequest::new();
    update.name = Some(Some(renamed.clone()));
    let updated = embedding_providers_api::update_embedding_provider(config, &created.id, update)
        .await
        .expect("update_embedding_provider should succeed");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, renamed);

    embedding_providers_api::delete_embedding_provider(config, &created.id)
        .await
        .expect("delete_embedding_provider should succeed");

    let after_delete = embedding_providers_api::get_embedding_provider(config, &created.id).await;
    match after_delete {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 after delete, got {err:?}"
        ),
        Ok(_) => panic!("get_embedding_provider should fail with 404 after delete"),
    }
}
