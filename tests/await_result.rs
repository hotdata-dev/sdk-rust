//! Exercise the public polling helper through the generated HTTP operation.
//! All requests use loopback with synthetic credentials.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use hotdata::apis::Error;
use hotdata::{AwaitResultError, Client, PollConfig};
use serde_json::{json, Value};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

// --- Shared transport fixtures ---

fn client(server: &MockServer) -> Client {
    Client::builder()
        .api_token("synthetic-test-token")
        .workspace_id("test-workspace")
        .base_url(server.uri())
        .reqwest_client(reqwest::Client::builder().no_proxy().build().unwrap())
        .build()
        .unwrap()
}

fn poll() -> PollConfig {
    PollConfig {
        timeout: Duration::from_secs(5),
        interval: Duration::from_millis(1),
    }
}

fn request() -> wiremock::MockBuilder {
    Mock::given(method("GET"))
        .and(path("/v1/results/result-1"))
        .and(header("X-Database-Id", "test-database"))
        .and(header("X-Workspace-Id", "test-workspace"))
        .and(header("Authorization", "Bearer synthetic-test-token"))
}

fn assert_failed(error: AwaitResultError, expected_message: Option<&str>) {
    match error {
        AwaitResultError::Failed {
            result_id,
            error_message,
        } => {
            assert_eq!(result_id, "result-1");
            assert_eq!(error_message.as_deref(), expected_message);
        }
        other => panic!("expected a persisted-result failure, got {other:?}"),
    }
}

async fn failed_body(body: Value, expected_message: Option<&str>) {
    let server = MockServer::start().await;
    request()
        .respond_with(ResponseTemplate::new(409).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;
    let error = client(&server)
        .await_result("result-1", "test-database", poll())
        .await
        .unwrap_err();
    assert_failed(error, expected_message);
}

// --- End shared transport fixtures ---

#[tokio::test]
async fn failed_result_409_preserves_server_message() {
    failed_body(
        json!({"result_id":"result-1", "status":"failed", "error_message":"persistence write failed"}),
        Some("persistence write failed"),
    )
    .await;
}

#[tokio::test]
async fn failed_result_409_allows_missing_message() {
    failed_body(json!({"result_id":"result-1", "status":"failed"}), None).await;
}

#[tokio::test]
async fn failed_result_409_allows_null_message() {
    failed_body(
        json!({"result_id":"result-1", "status":"failed", "error_message":null}),
        None,
    )
    .await;
}

#[tokio::test]
async fn processing_result_can_transition_to_failed_409() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let response_calls = Arc::clone(&calls);
    request()
        .respond_with(move |_: &Request| {
            if response_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                ResponseTemplate::new(202)
                    .set_body_json(json!({"result_id":"result-1", "status":"processing"}))
            } else {
                ResponseTemplate::new(409).set_body_json(json!({
                    "result_id":"result-1", "status":"failed", "error_message":"object storage unavailable"
                }))
            }
        })
        .expect(2)
        .mount(&server)
        .await;
    let error = client(&server)
        .await_result("result-1", "test-database", poll())
        .await
        .unwrap_err();
    assert_failed(error, Some("object storage unavailable"));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn ready_result_returns_original_rows() {
    let server = MockServer::start().await;
    request()
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result_id":"result-1", "status":"ready", "columns":["n"], "rows":[[42]], "row_count":1
        })))
        .expect(1)
        .mount(&server)
        .await;
    let result = client(&server)
        .await_result("result-1", "test-database", poll())
        .await
        .unwrap();
    assert_eq!(result.status, "ready");
    assert_eq!(result.result_id, "result-1");
    assert_eq!(result.row_count, Some(Some(1)));
    assert_eq!(result.rows.unwrap().unwrap()[0][0].as_json_str(), "42");
}

#[tokio::test]
async fn successful_http_failed_body_keeps_existing_classification() {
    let server = MockServer::start().await;
    request()
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result_id":"result-1", "status":"failed", "error_message":"legacy failure"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let error = client(&server)
        .await_result("result-1", "test-database", poll())
        .await
        .unwrap_err();
    assert_failed(error, Some("legacy failure"));
}

#[tokio::test]
async fn unrelated_or_malformed_errors_keep_http_status_and_body() {
    let cases = [
        (404, r#"{"error":"NOT_FOUND","message":"result missing"}"#),
        (409, r#"{"error":"CONFLICT","message":"database locked"}"#),
        (409, r#"{"result_id":"result-1","status":"ready"}"#),
        (
            500,
            r#"{"result_id":"result-1","status":"failed","error_message":"proxy failure"}"#,
        ),
        (409, r#"{"status":"#),
    ];
    for (status, body) in cases {
        let server = MockServer::start().await;
        request()
            .respond_with(ResponseTemplate::new(status).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;
        let error = client(&server)
            .await_result("result-1", "test-database", poll())
            .await
            .unwrap_err();
        match error {
            AwaitResultError::Api(Error::ResponseError(response)) => {
                assert_eq!(response.status.as_u16(), status);
                assert_eq!(response.content, body);
            }
            other => panic!("HTTP {status} must remain an API error, got {other:?}"),
        }
    }
}
