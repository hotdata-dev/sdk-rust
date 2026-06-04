//! Scenario: dataset_versioning.
//!
//! Create a dataset, exercise list_dataset_versions, pin to a specific version,
//! then confirm the pin is reflected. Confirms the versioning surface is
//! reachable and consistent.

mod common;

use hotdata::apis::datasets_api;
use hotdata::models;

fn inline_csv_source() -> models::DatasetSource {
    models::DatasetSource::DatasetSourceOneOf4(Box::new(models::DatasetSourceOneOf4::new(
        models::InlineData::new("a,b\n1,2\n3,4\n".to_string(), "csv".to_string()),
        models::dataset_source_one_of_4::Type::Inline,
    )))
}

#[tokio::test]
async fn dataset_versioning() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let label = common::sdkci_name("dataset-versioning");

    let created = datasets_api::create_dataset(
        config,
        models::CreateDatasetRequest::new(label, inline_csv_source()),
        None,
    )
    .await
    .expect("create_dataset should succeed");

    let versions = datasets_api::list_dataset_versions(config, &created.id, None, None)
        .await
        .expect("list_dataset_versions should succeed");
    assert_eq!(versions.dataset_id, created.id);
    assert!(versions.count >= 1, "expected at least one version");
    assert!(
        versions.versions.iter().any(|v| v.version == 1),
        "expected version 1 in {:?}",
        versions
            .versions
            .iter()
            .map(|v| v.version)
            .collect::<Vec<_>>()
    );

    let mut pin = models::UpdateDatasetRequest::new();
    pin.pinned_version = Some(Some(1));
    let pinned = datasets_api::update_dataset(config, &created.id, pin)
        .await
        .expect("update_dataset (pin) should succeed");
    assert_eq!(pinned.pinned_version, Some(Some(1)));
    assert!(pinned.latest_version >= 1);

    let fetched = datasets_api::get_dataset(config, &created.id)
        .await
        .expect("get_dataset should succeed");
    assert_eq!(fetched.pinned_version, Some(Some(1)));

    // Cleanup.
    let _ = datasets_api::delete_dataset(config, &created.id).await;
}
