//! Scenario: datasets_crud.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — create, read, list,
//! update, and delete a dataset; assert 404 after delete.

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
async fn datasets_crud() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let label = common::sdkci_name("datasets-crud");
    let new_label = format!("{label}-renamed");

    let created = datasets_api::create_dataset(
        config,
        models::CreateDatasetRequest::new(label.clone(), inline_csv_source()),
        None,
    )
    .await
    .expect("create_dataset should succeed");
    assert_eq!(created.label, label);
    assert!(!created.id.is_empty(), "created dataset must have an id");

    let fetched = datasets_api::get_dataset(config, &created.id)
        .await
        .expect("get_dataset should succeed");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.label, label);
    assert!(
        !fetched.columns.is_empty(),
        "expected inferred columns from inline CSV"
    );

    let listing = datasets_api::list_datasets(config, None, None)
        .await
        .expect("list_datasets should succeed");
    assert!(
        listing.datasets.iter().any(|d| d.id == created.id),
        "newly created dataset {} not present in list_datasets",
        created.id
    );

    let mut update = models::UpdateDatasetRequest::new();
    update.label = Some(Some(new_label.clone()));
    let updated = datasets_api::update_dataset(config, &created.id, update)
        .await
        .expect("update_dataset should succeed");
    assert_eq!(updated.label, new_label);

    // Delete and assert 404 afterwards.
    datasets_api::delete_dataset(config, &created.id)
        .await
        .expect("delete_dataset should succeed");

    let after_delete = datasets_api::get_dataset(config, &created.id).await;
    match after_delete {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 after delete, got {err:?}"
        ),
        Ok(_) => panic!("get_dataset should fail with 404 after delete"),
    }
}
