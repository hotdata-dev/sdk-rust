//! Shared HTTP retry helpers for the Hotdata Rust SDK.
//!
//! HTTP 429 (`OVERLOADED`) admission-shedding can hit any endpoint, not just
//! `POST /v1/query`: under concurrent load the server may shed a request with
//! HTTP 429 + `Retry-After` before it executes server-side. This module owns the
//! single retry implementation both paths use:
//!
//! * the generated free functions in `apis::*` (via the `api.mustache` template,
//!   which routes every op's `client.execute(req)` through [`execute_retrying`]),
//!   governed by [`Configuration::retry`](crate::apis::configuration::Configuration::retry);
//! * the hand-written enhanced query in [`crate::query`], which builds its own
//!   request to read the `Retry-After` header and reuses the [`backoff_delay`] /
//!   [`parse_retry_after`] / [`retry_after_secs`] primitives here so the two
//!   paths never drift.
//!
//! 429 retry on a POST is safe here: admission shedding happens before the
//! request executes, and the request bodies this SDK sends are buffered JSON that
//! [`reqwest::Request::try_clone`] clones cleanly. A non-clonable (streaming) body
//! degrades to a single attempt.
//!
//! This module is hand-written and listed in `.openapi-generator-ignore`, so it
//! survives client regeneration.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::StatusCode;

use crate::query::RetryPolicy;

/// HTTP 429: admission shedding (too many concurrent requests). Retry keys off
/// the status code since 429 is unambiguous and the body is not always parsed.
const HTTP_TOO_MANY_REQUESTS: StatusCode = StatusCode::TOO_MANY_REQUESTS;

/// Execute `req`, retrying on HTTP 429 (OVERLOADED admission-shedding) per
/// `retry`: honor `Retry-After` when present, else bounded exponential backoff
/// with jitter. Retries stop at `retry.max_retries` OR once the overall
/// `retry.deadline` budget would be exceeded — whichever comes first. The
/// request is cloned per attempt; a non-clonable (streaming) body degrades to a
/// single attempt.
///
/// When the budget or retry count is exhausted the last response (the 429) is
/// returned so the op's normal error mapping surfaces it to the caller — no new
/// error type. This mirrors `crate::query::submit_with_retry`, which enforces
/// the same `deadline` on the hand-written query path, so the two stay aligned.
pub(crate) async fn execute_retrying(
    client: &reqwest::Client,
    req: reqwest::Request,
    retry: &RetryPolicy,
) -> reqwest::Result<reqwest::Response> {
    let start = Instant::now();
    // attempt 0 is the initial request; 1..=max_retries are the retries.
    for attempt in 0..=retry.max_retries {
        // Clone the request before consuming it so a 429 can be retried. A
        // streaming body can't be cloned (`None`) — send it once with no retry.
        let Some(clone) = req.try_clone() else {
            return client.execute(req).await;
        };
        let resp = client.execute(clone).await?;
        if resp.status() != HTTP_TOO_MANY_REQUESTS || attempt == retry.max_retries {
            return Ok(resp);
        }
        // HTTP 429 OVERLOADED with attempts remaining: honor Retry-After when
        // present, else bounded exponential backoff with jitter.
        let delay = backoff_delay(retry, attempt + 1, parse_retry_after(&resp));
        // Stop if the deadline budget is already spent or this delay would push
        // total elapsed past it — max_backoff intentionally does not cap an
        // honored Retry-After, so the deadline is its only bound. Return the
        // 429 rather than sleeping past the budget.
        if start.elapsed() + delay > retry.deadline {
            return Ok(resp);
        }
        tokio::time::sleep(delay).await;
    }
    // Unreachable: the loop always returns on its last iteration (attempt ==
    // max_retries short-circuits above). Send once as a defensive fallback.
    client.execute(req).await
}

/// Delay before the next 429 retry: honor `Retry-After` when present (exactly,
/// uncapped, plus additive jitter so it is never below the server's value),
/// else bounded exponential backoff with jitter.
pub(crate) fn backoff_delay(
    retry: &RetryPolicy,
    attempt: u32,
    retry_after: Option<Duration>,
) -> Duration {
    if let Some(ra) = retry_after {
        // The server told us exactly how long to wait — honor it. Add jitter on
        // top (never below) to desync retries onto the freed slot, and do NOT
        // cap with max_backoff: capping would dishonor a Retry-After larger than
        // the cap. The overall deadline budget is the only bound.
        return ra + ra.mul_f64(retry.jitter * jitter_fraction());
    }
    let factor = 2f64.powi(attempt.saturating_sub(1) as i32);
    let base = retry.base_backoff.mul_f64(factor);
    let with_jitter = base.mul_f64(1.0 + retry.jitter * jitter_fraction());
    with_jitter.min(retry.max_backoff)
}

/// A pseudo-random fraction in `[0, 1)` for jitter. Derived from the wall clock
/// (no `rand` dependency); when `RetryPolicy::jitter` is 0 this value is
/// multiplied out, so timing is fully deterministic for tests.
fn jitter_fraction() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => (d.subsec_nanos() % 1_000) as f64 / 1_000.0,
        Err(_) => 0.0,
    }
}

/// Parse `Retry-After` (integer/float seconds form) into a [`Duration`]. The
/// HTTP-date form is not emitted by this API, so it is intentionally ignored.
pub(crate) fn parse_retry_after(resp: &reqwest::Response) -> Option<Duration> {
    resp.headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(retry_after_secs)
}

/// Parse a `Retry-After` header value (seconds form) into a [`Duration`].
///
/// Uses the fallible [`Duration::try_from_secs_f64`], which rejects negative,
/// non-finite (`inf`/`nan`), and overflowing values uniformly — so a malformed
/// or hostile server-supplied header degrades to normal backoff instead of
/// panicking inside the async retry path (`from_secs_f64` would panic on
/// `"inf"` or an overflowing value like `"1e30"`).
pub(crate) fn retry_after_secs(value: &str) -> Option<Duration> {
    let secs = value.trim().parse::<f64>().ok()?;
    Duration::try_from_secs_f64(secs).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A fast, deterministic retry policy: tiny backoffs, no jitter.
    fn fast_retry(max_retries: u32) -> RetryPolicy {
        RetryPolicy {
            max_retries,
            base_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(5),
            deadline: Duration::from_secs(30),
            jitter: 0.0,
        }
    }

    fn post_req(client: &reqwest::Client, url: &str) -> reqwest::Request {
        client
            .post(url)
            .json(&json!({"k": "v"}))
            .build()
            .expect("request should build")
    }

    #[tokio::test]
    async fn retries_two_429s_then_succeeds() {
        let server = MockServer::start().await;
        // Two 429s with Retry-After: 0, then a 200.
        Mock::given(method("POST"))
            .and(path("/thing"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
            .up_to_n_times(2)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/thing"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/thing", server.uri());
        let resp = execute_retrying(&client, post_req(&client, &url), &fast_retry(5))
            .await
            .expect("should succeed after retries");
        assert_eq!(resp.status(), StatusCode::OK);
        // 2 retried 429s + 1 success = 3 requests reached the server.
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn exhausts_after_max_retries() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/thing"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/thing", server.uri());
        // 1 initial + 2 retries = 3 requests, all 429; the final 429 is returned.
        let resp = execute_retrying(&client, post_req(&client, &url), &fast_retry(2))
            .await
            .expect("should return the final 429, not a transport error");
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn deadline_stops_retries_before_max_retries() {
        let server = MockServer::start().await;
        // Every response is a 429 with a Retry-After far larger than the budget.
        // max_backoff intentionally does NOT cap Retry-After, so only the
        // deadline can stop the loop — and it must, before max_retries is hit.
        Mock::given(method("POST"))
            .and(path("/thing"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "100"))
            .mount(&server)
            .await;

        let retry = RetryPolicy {
            max_retries: 10,
            base_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_secs(1),
            // Tiny budget: the first 100s Retry-After overshoots it immediately.
            deadline: Duration::from_millis(10),
            jitter: 0.0,
        };
        let client = reqwest::Client::new();
        let url = format!("{}/thing", server.uri());
        let resp = execute_retrying(&client, post_req(&client, &url), &retry)
            .await
            .expect("should return the 429 after the deadline stops retries");
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // The deadline stops retries after the very first 429 — well before the
        // 10 max_retries — so only one request reaches the server.
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn non_429_is_returned_without_retry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/thing"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/thing", server.uri());
        let resp = execute_retrying(&client, post_req(&client, &url), &fast_retry(5))
            .await
            .expect("should return the 400 without retrying");
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[test]
    fn retry_after_secs_parses_and_rejects_malformed() {
        assert_eq!(retry_after_secs("2"), Some(Duration::from_secs(2)));
        assert_eq!(retry_after_secs(" 1.5 "), Some(Duration::from_secs_f64(1.5)));
        assert_eq!(retry_after_secs("0"), Some(Duration::ZERO));
        // Malformed / hostile values must degrade to None, never panic.
        assert_eq!(retry_after_secs("inf"), None);
        assert_eq!(retry_after_secs("nan"), None);
        assert_eq!(retry_after_secs("1e30"), None);
        assert_eq!(retry_after_secs("-5"), None);
        assert_eq!(retry_after_secs("abc"), None);
        assert_eq!(retry_after_secs(""), None);
    }

    #[test]
    fn backoff_honors_retry_after_and_is_exponential() {
        let retry = RetryPolicy {
            base_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(100),
            jitter: 0.0,
            ..RetryPolicy::default()
        };
        // Retry-After honored exactly with jitter 0.
        assert_eq!(
            backoff_delay(&retry, 1, Some(Duration::from_secs(7))),
            Duration::from_secs(7)
        );
        // Otherwise exponential.
        assert_eq!(backoff_delay(&retry, 1, None), Duration::from_secs(1));
        assert_eq!(backoff_delay(&retry, 2, None), Duration::from_secs(2));
        assert_eq!(backoff_delay(&retry, 3, None), Duration::from_secs(4));
    }
}
