//! Typed, forward-compatible status enums for results and query runs.
//!
//! The generated models expose `status` as a bare `String` because the OpenAPI
//! spec types it as an open string. These hand-written enums give a typed,
//! ergonomic view without making the SDK brittle: each carries an
//! [`Other`](ResultStatus::Other) catch-all so an unrecognized status (e.g. a
//! value the server adds in a future release) round-trips instead of failing to
//! parse. This mirrors runtimedb's own lenient parsing (`ResultStatus::parse` /
//! `QueryRunStatus::parse`, which fall back rather than reject) — a generated,
//! closed enum would instead break deserialization on any new status.
//!
//! This module is hand-written and regeneration-immune; it never edits the
//! generated models, only interprets their `status` strings.
//!
//! ```
//! use hotdata::{ResultStatus, ResultStatusExt};
//! use hotdata::models::GetResultResponse;
//!
//! fn describe(resp: &GetResultResponse) -> bool {
//!     matches!(resp.result_status(), ResultStatus::Ready)
//! }
//! ```

use crate::models;

/// Status of a persisted query result (`GET /v1/results/{id}`).
///
/// Mirrors runtimedb's `ResultStatus`. Unknown wire values are preserved as
/// [`ResultStatus::Other`] rather than rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultStatus {
    /// Reserved but not started yet.
    Pending,
    /// Query executing or persistence in progress.
    Processing,
    /// Ready for retrieval.
    Ready,
    /// Query or persistence failed.
    Failed,
    /// A status not recognized by this SDK version (forward-compatibility).
    Other(String),
}

impl ResultStatus {
    /// Parse a status string. Unknown values become [`ResultStatus::Other`].
    pub fn parse(s: &str) -> Self {
        match s {
            "pending" => ResultStatus::Pending,
            "processing" => ResultStatus::Processing,
            "ready" => ResultStatus::Ready,
            "failed" => ResultStatus::Failed,
            other => ResultStatus::Other(other.to_owned()),
        }
    }

    /// The wire representation of this status.
    pub fn as_str(&self) -> &str {
        match self {
            ResultStatus::Pending => "pending",
            ResultStatus::Processing => "processing",
            ResultStatus::Ready => "ready",
            ResultStatus::Failed => "failed",
            ResultStatus::Other(s) => s,
        }
    }

    /// True if the result is ready for retrieval.
    pub fn is_ready(&self) -> bool {
        matches!(self, ResultStatus::Ready)
    }

    /// True if the result failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, ResultStatus::Failed)
    }
}

impl std::fmt::Display for ResultStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ResultStatus {
    fn from(s: &str) -> Self {
        ResultStatus::parse(s)
    }
}

/// Status of a query run (`GET /v1/query-runs/{id}`).
///
/// Mirrors runtimedb's `QueryRunStatus`. Unknown wire values are preserved as
/// [`QueryRunStatus::Other`] rather than rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryRunStatus {
    /// The query is still executing.
    Running,
    /// The query finished successfully.
    Succeeded,
    /// The query failed.
    Failed,
    /// A status not recognized by this SDK version (forward-compatibility).
    Other(String),
}

impl QueryRunStatus {
    /// Parse a status string. Unknown values become [`QueryRunStatus::Other`].
    pub fn parse(s: &str) -> Self {
        match s {
            "running" => QueryRunStatus::Running,
            "succeeded" => QueryRunStatus::Succeeded,
            "failed" => QueryRunStatus::Failed,
            other => QueryRunStatus::Other(other.to_owned()),
        }
    }

    /// The wire representation of this status.
    pub fn as_str(&self) -> &str {
        match self {
            QueryRunStatus::Running => "running",
            QueryRunStatus::Succeeded => "succeeded",
            QueryRunStatus::Failed => "failed",
            QueryRunStatus::Other(s) => s,
        }
    }

    /// True once the run has reached a terminal state (`succeeded` or `failed`).
    /// An unrecognized status is treated as non-terminal so callers keep polling.
    pub fn is_terminal(&self) -> bool {
        matches!(self, QueryRunStatus::Succeeded | QueryRunStatus::Failed)
    }

    /// True if the run succeeded.
    pub fn is_succeeded(&self) -> bool {
        matches!(self, QueryRunStatus::Succeeded)
    }
}

impl std::fmt::Display for QueryRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for QueryRunStatus {
    fn from(s: &str) -> Self {
        QueryRunStatus::parse(s)
    }
}

/// Typed access to a response's result `status` string.
pub trait ResultStatusExt {
    /// Interpret the `status` field as a [`ResultStatus`].
    fn result_status(&self) -> ResultStatus;
}

impl ResultStatusExt for models::GetResultResponse {
    fn result_status(&self) -> ResultStatus {
        ResultStatus::parse(&self.status)
    }
}

impl ResultStatusExt for models::ResultInfo {
    fn result_status(&self) -> ResultStatus {
        ResultStatus::parse(&self.status)
    }
}

/// Typed access to a response's query-run `status` string.
pub trait QueryRunStatusExt {
    /// Interpret the `status` field as a [`QueryRunStatus`].
    fn run_status(&self) -> QueryRunStatus;
}

impl QueryRunStatusExt for models::QueryRunInfo {
    fn run_status(&self) -> QueryRunStatus {
        QueryRunStatus::parse(&self.status)
    }
}

impl QueryRunStatusExt for models::AsyncQueryResponse {
    fn run_status(&self) -> QueryRunStatus {
        QueryRunStatus::parse(&self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_status_round_trips_known_values() {
        for s in ["pending", "processing", "ready", "failed"] {
            assert_eq!(ResultStatus::parse(s).as_str(), s);
        }
        assert!(ResultStatus::parse("ready").is_ready());
        assert!(ResultStatus::parse("failed").is_failed());
    }

    #[test]
    fn result_status_preserves_unknown() {
        let s = ResultStatus::parse("quantum");
        assert_eq!(s, ResultStatus::Other("quantum".to_owned()));
        assert_eq!(s.as_str(), "quantum");
        assert!(!s.is_ready() && !s.is_failed());
    }

    #[test]
    fn query_run_status_terminality() {
        assert!(QueryRunStatus::parse("succeeded").is_terminal());
        assert!(QueryRunStatus::parse("failed").is_terminal());
        assert!(!QueryRunStatus::parse("running").is_terminal());
        // Unknown statuses are non-terminal so polling continues.
        assert!(!QueryRunStatus::parse("paused").is_terminal());
        assert!(QueryRunStatus::parse("succeeded").is_succeeded());
    }

    #[test]
    fn from_str_impls() {
        assert_eq!(ResultStatus::from("ready"), ResultStatus::Ready);
        assert_eq!(QueryRunStatus::from("running"), QueryRunStatus::Running);
    }
}
