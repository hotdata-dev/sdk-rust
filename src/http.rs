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

use std::error::Error as StdError;
use std::io;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::StatusCode;

use crate::query::RetryPolicy;

/// HTTP 429: admission shedding (too many concurrent requests). Retry keys off
/// the status code since 429 is unambiguous and the body is not always parsed.
const HTTP_TOO_MANY_REQUESTS: StatusCode = StatusCode::TOO_MANY_REQUESTS;

/// Classify a [`reqwest::Error`] as a **pre-response connection error** — a
/// transport failure that happened *before any response bytes were received*, so
/// the server did no work and a retry cannot double-execute. Safe to retry on
/// **any** method, including `POST` (cf. hotdata-dev/sdk-rust#63,
/// hotdata-dev/sdk-python#118).
///
/// Two classes qualify:
///
/// * **Connect-phase failures** ([`reqwest::Error::is_connect`]): the connection
///   was never established (DNS / TCP connect / TLS), so the request never left
///   the client.
/// * **Send-phase connection resets**: a pooled keep-alive socket that an
///   intermediary (load balancer / reverse proxy) closed on its idle timeout
///   surfaces, on the next reuse, as a `ConnectionReset` / `ConnectionAborted` /
///   `BrokenPipe` `io::Error` (or an `UnexpectedEof` before the status line)
///   while sending the request. The request never reached the server.
///
/// Errors that imply a response was already in flight are deliberately excluded:
/// [`is_body`](reqwest::Error::is_body), [`is_decode`](reqwest::Error::is_decode),
/// and [`is_status`](reqwest::Error::is_status) all mean the request reached the
/// server, so retrying a non-idempotent `POST` there could double-execute. Those
/// stay caller-driven / idempotent-only, exactly as #63 scopes it.
pub(crate) fn is_pre_response_transport_error(err: &reqwest::Error) -> bool {
    // A response was (at least partially) received — not pre-response.
    if err.is_body() || err.is_decode() || err.is_status() {
        return false;
    }
    // Connection establishment failed: the request never left the client.
    if err.is_connect() {
        return true;
    }
    // Otherwise look for a connection-level I/O error in the source chain. A
    // stale pooled socket reset on reuse lands here (kind ConnectionReset on the
    // request send), distinct from a connect-phase failure.
    let mut source: Option<&(dyn StdError + 'static)> = err.source();
    while let Some(e) = source {
        if let Some(io_err) = e.downcast_ref::<io::Error>() {
            return matches!(
                io_err.kind(),
                io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::BrokenPipe
                    | io::ErrorKind::UnexpectedEof
            );
        }
        source = e.source();
    }
    false
}

/// Execute `req`, retrying on HTTP 429 (OVERLOADED admission-shedding) **and on
/// pre-response connection errors** (stale keep-alive resets — see
/// [`is_pre_response_transport_error`]) per `retry`: honor `Retry-After` when
/// present (429 only), else bounded exponential backoff with jitter. Retries
/// stop at `retry.max_retries` OR once the overall `retry.deadline` budget would
/// be exceeded — whichever comes first. The request is cloned per attempt; a
/// non-clonable (streaming) body degrades to a single attempt.
///
/// A pre-response connection error is safe to retry on any method (the request
/// never reached the server); response-phase transport errors are *not* retried
/// here, so a non-idempotent `POST` can't double-execute.
///
/// When the budget or retry count is exhausted the last response (the 429) is
/// returned, or the last transport error is propagated, so the op's normal error
/// mapping surfaces it to the caller — no new error type. This mirrors
/// `crate::query::submit_with_retry`, which enforces the same `deadline` on the
/// hand-written query path, so the two stay aligned.
pub(crate) async fn execute_retrying(
    client: &reqwest::Client,
    req: reqwest::Request,
    retry: &RetryPolicy,
) -> reqwest::Result<reqwest::Response> {
    let start = Instant::now();
    // attempt 0 is the initial request; 1..=max_retries are the retries.
    for attempt in 0..=retry.max_retries {
        // Clone the request before consuming it so a 429 or a pre-response reset
        // can be retried. A streaming body can't be cloned (`None`) — send it
        // once with no retry.
        let Some(clone) = req.try_clone() else {
            return client.execute(req).await;
        };
        let resp = match client.execute(clone).await {
            Ok(resp) => resp,
            Err(e) => {
                // Pre-response connection reset (e.g. a stale pooled keep-alive
                // socket) with attempts remaining and budget left: retry on a
                // fresh connection. Anything else (or budget/count exhausted)
                // propagates unchanged.
                if attempt == retry.max_retries || !is_pre_response_transport_error(&e) {
                    return Err(e);
                }
                let delay = backoff_delay(retry, attempt + 1, None);
                if start.elapsed() + delay > retry.deadline {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
                continue;
            }
        };
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
    #[cfg(unix)]
    use std::io::{Read, Write};
    #[cfg(unix)]
    use std::net::TcpListener;
    #[cfg(unix)]
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[cfg(unix)]
    use std::sync::Arc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Force a TCP RST on close by setting `SO_LINGER` to 0. `std`'s
    /// `TcpStream::set_linger` is still unstable, so go through `setsockopt`
    /// directly (test-only, `unix`-only). A reset, not a graceful FIN, is the
    /// stale keep-alive symptom #63 targets (hyper surfaces it as a
    /// `ConnectionReset` `io::Error`, distinct from an `IncompleteMessage`).
    #[cfg(unix)]
    fn force_rst_on_close(fd: i32) {
        #[repr(C)]
        struct Linger {
            l_onoff: i32,
            l_linger: i32,
        }
        extern "C" {
            fn setsockopt(
                s: i32,
                level: i32,
                name: i32,
                val: *const core::ffi::c_void,
                len: u32,
            ) -> i32;
        }
        #[cfg(target_os = "linux")]
        let (sol_socket, so_linger) = (1i32, 13i32);
        #[cfg(not(target_os = "linux"))]
        let (sol_socket, so_linger) = (0xffffi32, 0x0080i32); // macOS / BSD
        let l = Linger {
            l_onoff: 1,
            l_linger: 0,
        };
        unsafe {
            setsockopt(
                fd,
                sol_socket,
                so_linger,
                &l as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<Linger>() as u32,
            );
        }
    }

    /// Spawn a bare TCP server that resets the first `reset_count` connections
    /// before any response (forcing a `ConnectionReset` via `SO_LINGER` 0 — the
    /// stale keep-alive symptom from #63), then answers `200 OK` with a tiny
    /// JSON body. Returns the base URL and a counter of accepted connections so
    /// a test can assert how many attempts reached the wire.
    #[cfg(unix)]
    fn reset_then_ok_server(reset_count: usize) -> (String, Arc<AtomicUsize>) {
        use std::os::unix::io::AsRawFd;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let conns = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&conns);
        std::thread::spawn(move || {
            let mut i = 0usize;
            for stream in listener.incoming() {
                let Ok(mut s) = stream else { continue };
                counter.fetch_add(1, Ordering::SeqCst);
                // Drain the client's request bytes so it finishes writing before
                // we act (otherwise the RST can race the request send).
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                if i < reset_count {
                    force_rst_on_close(s.as_raw_fd());
                    drop(s);
                } else {
                    let body = br#"{"ok":true}"#;
                    let head = format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                         content-length: {}\r\nconnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = s.write_all(head.as_bytes());
                    let _ = s.write_all(body);
                    let _ = s.flush();
                }
                i += 1;
            }
        });
        (format!("http://{addr}"), conns)
    }

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

    #[cfg(unix)]
    #[tokio::test]
    async fn retries_pre_response_reset_then_succeeds() {
        // First connection is reset before any response (stale keep-alive
        // symptom); the retry on a fresh connection gets a 200. A POST must be
        // retried here — the request never reached the server.
        let (base, conns) = reset_then_ok_server(1);
        let client = reqwest::Client::new();
        let req = client
            .post(format!("{base}/thing"))
            .json(&json!({"k": "v"}))
            .build()
            .expect("request should build");
        let resp = execute_retrying(&client, req, &fast_retry(5))
            .await
            .expect("pre-response reset should be retried, then succeed");
        assert_eq!(resp.status(), StatusCode::OK);
        // 1 reset + 1 success = 2 connections reached the wire.
        assert_eq!(conns.load(Ordering::SeqCst), 2);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn pre_response_reset_propagates_after_max_retries() {
        // Every connection is reset: retries are exhausted and the transport
        // error propagates (no new error type, mirroring the 429 path).
        let (base, conns) = reset_then_ok_server(usize::MAX);
        let client = reqwest::Client::new();
        let req = client
            .post(format!("{base}/thing"))
            .json(&json!({"k": "v"}))
            .build()
            .expect("request should build");
        let err = execute_retrying(&client, req, &fast_retry(2))
            .await
            .expect_err("persistent reset should propagate after retries");
        assert!(is_pre_response_transport_error(&err));
        // 1 initial + 2 retries = 3 connections, all reset.
        assert_eq!(conns.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn connect_failure_is_pre_response() {
        // A refused connection (nothing listening) never reaches the server, so
        // it classifies as a pre-response error retryable on any method.
        let client = reqwest::Client::new();
        let err = client
            .post("http://127.0.0.1:1/thing")
            .json(&json!({"k": "v"}))
            .send()
            .await
            .expect_err("connect to port 1 should fail");
        assert!(err.is_connect());
        assert!(is_pre_response_transport_error(&err));
    }

    #[test]
    fn retry_after_secs_parses_and_rejects_malformed() {
        assert_eq!(retry_after_secs("2"), Some(Duration::from_secs(2)));
        assert_eq!(
            retry_after_secs(" 1.5 "),
            Some(Duration::from_secs_f64(1.5))
        );
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
