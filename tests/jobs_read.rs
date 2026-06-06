//! Scenario: jobs_read.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — read-only. list_jobs
//! returns the workspace job history, and get_job fetches a single job by id
//! (the first from the list, when any exist). Never starts a job; tolerates an
//! empty history.

mod common;

use hotdata::apis::jobs_api;

#[tokio::test]
async fn jobs_read() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let listing = jobs_api::list_jobs(config, None, None, Some(10), Some(0))
        .await
        .expect("list_jobs should succeed");

    let Some(first) = listing.jobs.first() else {
        eprintln!("jobs_read: workspace has no job history; list_jobs returned empty (tolerated)");
        return;
    };

    // Point lookup of the first listed job must round-trip the id.
    let job = jobs_api::get_job(config, &first.id)
        .await
        .expect("get_job should succeed");
    assert_eq!(job.id, first.id);
    assert_eq!(
        job.job_type, first.job_type,
        "get_job should agree with list_jobs on job_type"
    );

    // A fabricated job id must not resolve.
    let bogus = jobs_api::get_job(config, "sdkci-no-such-job").await;
    match bogus {
        Err(err) => {
            let status = common::status_of(&err);
            assert!(
                matches!(status, Some(404) | Some(400)),
                "expected 404/400 for an unknown job id, got {status:?} ({err:?})"
            );
        }
        Ok(_) => panic!("get_job should fail for an unknown job id"),
    }
}
