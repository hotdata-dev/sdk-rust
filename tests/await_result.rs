//! Exercise the public polling helper through the generated HTTP operation.
//! All requests use loopback with synthetic credentials.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use hotdata::apis::Error;
use hotdata::models::QueryRequest;
use hotdata::{AwaitResultError, Client, PollConfig, QueryError, ResultError};
use serde_json::{json, Value};
use wiremock::matchers::{header, method, path, query_param};
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
    failed_response(
        ResponseTemplate::new(409).set_body_json(body),
        expected_message,
    )
    .await;
}

async fn failed_response(response: ResponseTemplate, expected_message: Option<&str>) {
    let server = MockServer::start().await;
    request()
        .respond_with(response)
        .expect(1)
        .mount(&server)
        .await;
    let error = client(&server)
        .await_result("result-1", "test-database", poll())
        .await
        .unwrap_err();
    assert_failed(error, expected_message);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
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
async fn failed_result_409_uses_requested_id() {
    failed_body(
        json!({"result_id":"different-response-id", "status":"failed", "error_message":"write failed"}),
        Some("write failed"),
    )
    .await;
}

#[tokio::test]
async fn empty_body_409_is_failed_without_message() {
    failed_response(ResponseTemplate::new(409), None).await;
}

#[tokio::test]
async fn nonfailed_or_malformed_409_is_failed_without_message() {
    for body in [
        r#"{"error":"CONFLICT","message":"database locked"}"#,
        r#"{"error":"CONFLICT","message":"database locked","error_message":"untyped message"}"#,
        r#"{"result_id":"result-1","status":"ready"}"#,
        r#"{"result_id":"result-1","status":"processing"}"#,
        r#"{"result_id":"result-1","status":"unexpected"}"#,
        r#"{"status":"#,
    ] {
        failed_response(
            ResponseTemplate::new(409).set_body_raw(body, "application/json"),
            None,
        )
        .await;
    }
}

#[tokio::test]
async fn nonfailed_409_preserves_typed_message() {
    failed_body(
        json!({"result_id":"different-response-id", "status":"ready", "error_message":"write failed"}),
        Some("write failed"),
    )
    .await;
}

#[tokio::test]
async fn pending_or_processing_result_can_transition_to_failed_409() {
    for initial_status in ["pending", "processing"] {
        let server = MockServer::start().await;
        let calls = Arc::new(AtomicUsize::new(0));
        let response_calls = Arc::clone(&calls);
        request()
            .respond_with(move |_: &Request| {
                if response_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    ResponseTemplate::new(202)
                        .set_body_json(json!({"result_id":"result-1", "status":initial_status}))
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
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }
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
async fn non_conflict_errors_keep_http_status_and_body() {
    let cases = [
        (404, r#"{"error":"NOT_FOUND","message":"result missing"}"#),
        (
            500,
            r#"{"result_id":"result-1","status":"failed","error_message":"proxy failure"}"#,
        ),
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

#[tokio::test]
async fn query_auto_follow_preserves_result_error_contract() {
    let cases = [
        (
            409,
            r#"{"result_id":"different-response-id","status":"failed","error_message":"write failed"}"#,
            Some("write failed"),
        ),
        (409, r#"{"result_id":"result-1","status":"failed"}"#, None),
        (
            409,
            r#"{"result_id":"result-1","status":"failed","error_message":null}"#,
            None,
        ),
        (
            409,
            r#"{"result_id":"different-response-id","status":"ready","error_message":"write failed"}"#,
            Some("write failed"),
        ),
        (409, r#"{"result_id":"result-1","status":"ready"}"#, None),
        (
            409,
            r#"{"error":"CONFLICT","message":"database locked","error_message":"untyped message"}"#,
            None,
        ),
        (409, r#"{"status":"#, None),
        (409, "", None),
        (
            404,
            r#"{"error":"NOT_FOUND","message":"result missing"}"#,
            None,
        ),
        (
            500,
            r#"{"result_id":"result-1","status":"failed","error_message":"proxy failure"}"#,
            None,
        ),
    ];
    for (status, body, expected_message) in cases {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .and(header("X-Database-Id", "test-database"))
            .and(header("X-Workspace-Id", "test-workspace"))
            .and(header("Authorization", "Bearer synthetic-test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "columns":["n"], "execution_time_ms":1, "nullable":[false],
                "preview_row_count":1, "query_run_id":"query-run-1", "result_id":"result-1",
                "row_count":1, "rows":[[42]], "truncated":true, "total_row_count":2
            })))
            .expect(1)
            .mount(&server)
            .await;
        request()
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(status).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let error = client(&server)
            .query_in(
                QueryRequest::new("SELECT 42 AS n".to_owned()),
                "test-database",
            )
            .await
            .unwrap_err();
        match error {
            QueryError::Result(ResultError::Failed {
                result_id,
                error_message,
            }) if status == 409 => {
                assert_eq!(result_id, "result-1");
                assert_eq!(error_message.as_deref(), expected_message);
            }
            QueryError::Poll(Error::ResponseError(response)) if status != 409 => {
                assert_eq!(response.status.as_u16(), status);
                assert_eq!(response.content, body);
            }
            other => panic!("unexpected query auto-follow error for HTTP {status}: {other:?}"),
        }
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }
}
