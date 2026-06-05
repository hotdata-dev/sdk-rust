//! Request/response debug logging for the Hotdata Rust SDK.
//!
//! Every outgoing HTTP call in this crate funnels its request and response
//! through this module: the generated free functions in `apis::*` (via the
//! `api.mustache` template), and the hand-written ergonomic layer —
//! [`Client::submit_query`](crate::client::Client::submit_query),
//! [`Client::upload_stream`](crate::client::Client::upload_stream), the Arrow
//! result fetch in [`crate::arrow`], and the API-token -> JWT mint in
//! [`crate::auth`]. Each emits `log::debug!` records on the [`TARGET`]
//! (`hotdata::http`) target so a host can switch them on with any `log` backend
//! and render them however it likes (e.g. the CLI's `--debug` flag, which maps
//! this target to its `>>> METHOD url` / `<<< status` output).
//!
//! The SDK itself installs no logger and prints nothing on its own — these are
//! plain `log` facade records. With no backend, or with the target filtered
//! out, the cost is a single atomic load per call (see [`log_request`]); the
//! bodies are only stringified/redacted when the target is actually enabled.
//!
//! **Redaction.** Sensitive material is masked before it ever reaches the log
//! facade, mirroring the CLI's `mask_credential`: `Authorization` bearer tokens
//! are masked (scheme preserved), and known-sensitive JSON object keys / form
//! fields (`api_token`, `refresh_token`, `secret`, `password`, …) have their
//! values masked in place — recursively, so nested bodies are covered.
//!
//! This module is hand-written and listed in `.openapi-generator-ignore`, so it
//! survives client regeneration; the `api.mustache` template emits the
//! `crate::http_log::*` calls at every generated op.

use log::{debug, log_enabled, Level};

/// `log` target for every HTTP debug record this crate emits. Hosts filter on
/// this to route the SDK's wire logs (e.g. the CLI maps it to its `--debug`
/// output) without picking up unrelated `log` traffic.
pub const TARGET: &str = "hotdata::http";

/// JSON object keys and `x-www-form-urlencoded` field names whose values are
/// masked before logging. Compared case-insensitively. Mirrors the credentials
/// the CLI redacts so SDK and CLI debug output stay consistent.
const SENSITIVE_KEYS: &[&str] = &[
    "authorization",
    "api_token",
    "access_token",
    "refresh_token",
    "token",
    "client_secret",
    "secret",
    "secret_value",
    "password",
    "passwd",
    "private_key",
    "credentials",
    "connection_string",
];

/// Cap on the rendered length of a non-JSON, non-form body so a stray large or
/// binary-ish payload can't flood the log.
const MAX_BODY_LEN: usize = 4096;

/// Whether the HTTP debug target is currently enabled. Call sites guard on this
/// (cheaply) before doing any redaction work.
fn enabled() -> bool {
    log_enabled!(target: TARGET, Level::Debug)
}

/// Mask a credential to its first + last 4 characters (`XXXX...YYYY`), or `***`
/// if it is too short to reveal anything safely. Mirrors the CLI's
/// `mask_credential` so SDK and CLI debug logs read identically; the visible
/// tail makes it easy to tell which token is on the wire.
pub fn mask_credential(s: &str) -> String {
    if s.len() >= 12 {
        format!("{}...{}", &s[..4], &s[s.len() - 4..])
    } else if s.len() > 4 {
        // Short-ish: show the head but no tail, so we don't reveal most of it.
        format!("{}...", &s[..4])
    } else {
        "***".into()
    }
}

/// Whether a JSON key / form field name names sensitive material to mask.
fn is_sensitive(key: &str) -> bool {
    SENSITIVE_KEYS.iter().any(|k| key.eq_ignore_ascii_case(k))
}

/// Mask an `Authorization` header value, preserving the scheme prefix
/// (`Bearer`, `Basic`, …) so the log still reads sensibly.
fn mask_auth_value(value: &str) -> String {
    if let Some(token) = value.strip_prefix("Bearer ") {
        format!("Bearer {}", mask_credential(token))
    } else {
        mask_credential(value)
    }
}

/// Log an outgoing request: `>>> METHOD url`, each header (with `Authorization`
/// masked), and the request body with sensitive fields redacted.
///
/// Called after `req_builder.build()?` and before `client.execute(req)` (which
/// consumes the request). Streaming bodies (file/byte-stream uploads) report
/// their kind rather than buffering — `reqwest::Body::as_bytes` only yields the
/// in-memory bodies (`.json(..)` / `.form(..)` / `.body(bytes)`).
pub fn log_request(req: &reqwest::Request) {
    if !enabled() {
        return;
    }
    debug!(target: TARGET, ">>> {} {}", req.method(), req.url());
    for (name, value) in req.headers() {
        let key = name.as_str();
        let shown = match value.to_str() {
            Ok(v) if key.eq_ignore_ascii_case("authorization") => mask_auth_value(v),
            Ok(v) => v.to_string(),
            Err(_) => "<non-utf8>".to_string(),
        };
        debug!(target: TARGET, "  {key}: {shown}");
    }
    match req.body().and_then(reqwest::Body::as_bytes) {
        Some(bytes) if !bytes.is_empty() => debug!(target: TARGET, "{}", redact_body(bytes)),
        Some(_) => {}
        // No in-memory body: either a bodyless request or a streamed upload.
        None if req.body().is_some() => debug!(target: TARGET, "[streaming body]"),
        None => {}
    }
}

/// Log a response status line: `<<< 200 OK`.
///
/// Emitted once per call, right after the status is read, so it covers every
/// branch (including empty-body and streamed-body responses) uniformly.
pub fn log_response_status(status: reqwest::StatusCode) {
    if !enabled() {
        return;
    }
    debug!(
        target: TARGET,
        "<<< {} {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or("")
    );
}

/// Log a response body with sensitive fields redacted. No-op for an empty body.
pub fn log_response_body(body: &str) {
    if !enabled() || body.is_empty() {
        return;
    }
    debug!(target: TARGET, "{}", redact_body(body.as_bytes()));
}

/// Render a request/response body for logging with sensitive values masked.
///
/// JSON bodies are parsed and masked structurally (recursing into nested
/// objects/arrays). A non-JSON body is treated as `x-www-form-urlencoded` when
/// it parses as `k=v(&k=v)*` (the shape of the auth mint body) and its
/// sensitive fields are masked; anything else is shown verbatim, length-capped.
fn redact_body(bytes: &[u8]) -> String {
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return format!("[binary: {} bytes]", bytes.len()),
    };

    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(text) {
        redact_json(&mut value);
        return serde_json::to_string(&value).unwrap_or_else(|_| truncate(text));
    }

    if let Some(form) = redact_form(text) {
        return form;
    }

    truncate(text)
}

/// Recursively mask the values of sensitive keys in a JSON value, in place.
fn redact_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if is_sensitive(key) {
                    if let Some(s) = val.as_str() {
                        *val = serde_json::Value::String(mask_credential(s));
                    }
                } else {
                    redact_json(val);
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(redact_json),
        _ => {}
    }
}

/// Mask sensitive fields in an `x-www-form-urlencoded` body, returning `None`
/// if `text` doesn't look like one (so the caller can fall back to verbatim).
///
/// "Looks like a form" means every `&`-separated segment is a non-empty `key=…`
/// pair — true for the SDK's only non-JSON in-memory body (the token mint), and
/// false for arbitrary prose, which then logs verbatim.
fn redact_form(text: &str) -> Option<String> {
    let segments: Vec<&str> = text.split('&').collect();
    let looks_like_form = segments.iter().all(|seg| {
        seg.split_once('=')
            .is_some_and(|(k, _)| !k.is_empty() && !k.contains(char::is_whitespace))
    });
    if !looks_like_form {
        return None;
    }
    let redacted = segments
        .iter()
        .map(|seg| match seg.split_once('=') {
            Some((k, v)) if is_sensitive(k) => format!("{k}={}", mask_credential(v)),
            _ => seg.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&");
    Some(redacted)
}

/// Truncate an over-long body for logging, appending an elision marker.
fn truncate(text: &str) -> String {
    if text.len() <= MAX_BODY_LEN {
        return text.to_string();
    }
    // Cut on a char boundary at or below the cap.
    let mut end = MAX_BODY_LEN;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… [{} bytes total]", &text[..end], text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_credential_long_shows_head_and_tail() {
        assert_eq!(mask_credential("abcdefghijkl"), "abcd...ijkl");
        assert_eq!(
            mask_credential("hd_0123456789abcdef"),
            "hd_0...cdef"
        );
    }

    #[test]
    fn mask_credential_medium_shows_head_only() {
        // 5..=11 chars: head only, no tail.
        assert_eq!(mask_credential("abcdef"), "abcd...");
    }

    #[test]
    fn mask_credential_short_is_fully_hidden() {
        assert_eq!(mask_credential("abcd"), "***");
        assert_eq!(mask_credential(""), "***");
    }

    #[test]
    fn mask_auth_preserves_bearer_scheme() {
        assert_eq!(
            mask_auth_value("Bearer eyJhbGciOiJIUzI1NiJ9.payload.signature"),
            "Bearer eyJh...ture"
        );
        // Non-bearer values are masked whole.
        assert_eq!(mask_auth_value("Basic dXNlcjpwYXNz"), "Basi...YXNz");
    }

    #[test]
    fn json_body_masks_sensitive_keys_recursively() {
        let body = serde_json::json!({
            "name": "prod-db",
            "secret": "supersecretvalue123",
            "nested": { "api_token": "hd_abcdef0123456789", "keep": "visible" },
            "list": [ { "password": "hunter2hunter2" } ]
        })
        .to_string();
        let out = redact_body(body.as_bytes());
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["name"], "prod-db");
        assert_eq!(v["secret"], "supe...e123");
        assert_eq!(v["nested"]["api_token"], "hd_a...6789");
        assert_eq!(v["nested"]["keep"], "visible");
        assert_eq!(v["list"][0]["password"], "hunt...ter2");
        // The raw secret never appears in the rendered output.
        assert!(!out.contains("supersecretvalue123"));
        assert!(!out.contains("hd_abcdef0123456789"));
    }

    #[test]
    fn form_body_masks_sensitive_fields() {
        let body = "grant_type=api_token&api_token=hd_0123456789abcdef&client_id=hotdata-rust-sdk";
        let out = redact_body(body.as_bytes());
        assert!(out.contains("grant_type=api_token"));
        assert!(out.contains("client_id=hotdata-rust-sdk"));
        assert!(out.contains("api_token=hd_0...cdef"));
        assert!(!out.contains("hd_0123456789abcdef"));
    }

    #[test]
    fn non_form_text_is_logged_verbatim() {
        // Plain prose isn't mistaken for a form (no spurious masking/mangling).
        let body = "this is not a form body";
        assert_eq!(redact_body(body.as_bytes()), body);
    }

    #[test]
    fn binary_body_reports_byte_count() {
        let out = redact_body(&[0xff, 0xfe, 0x00, 0x01]);
        assert_eq!(out, "[binary: 4 bytes]");
    }

    #[test]
    fn overlong_plain_body_is_truncated() {
        let body = "x".repeat(MAX_BODY_LEN + 100);
        let out = redact_body(body.as_bytes());
        assert!(out.len() < body.len());
        assert!(out.contains("bytes total]"));
    }
}
