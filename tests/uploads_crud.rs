//! Scenario: uploads_crud.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — upload a small
//! `sdkci-*` file via upload_file, then confirm it appears in list_uploads.
//! There is no delete-upload endpoint, so server-side orphans are reclaimed by
//! the nightly `sdkci-*` sweep rather than torn down here; we only clean up the
//! local temp file.

mod common;

use hotdata::apis::uploads_api;

#[tokio::test]
async fn uploads_crud() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    // Write a small CSV to a temp file named `sdkci-*` so any orphan is
    // identifiable to the sweep.
    let contents = b"a,b\n1,2\n3,4\n";
    let path = std::env::temp_dir().join(format!("{}.csv", common::sdkci_name("uploads-crud")));
    std::fs::write(&path, contents).expect("writing the temp upload file should succeed");

    let uploaded = uploads_api::upload_file(config, path.clone()).await;
    // Always remove the local temp file, whatever the upload did.
    let _ = std::fs::remove_file(&path);
    let uploaded = uploaded.expect("upload_file should succeed");

    assert!(!uploaded.id.is_empty(), "upload must return an id");
    assert_eq!(
        uploaded.size_bytes as usize,
        contents.len(),
        "reported size_bytes should match the uploaded file"
    );
    assert!(!uploaded.status.is_empty(), "upload must report a status");

    let listing = uploads_api::list_uploads(config, None)
        .await
        .expect("list_uploads should succeed");
    assert!(
        listing.uploads.iter().any(|u| u.id == uploaded.id),
        "uploaded file {} not present in list_uploads",
        uploaded.id
    );
}
