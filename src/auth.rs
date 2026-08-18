//! Pluggable per-request bearer credentials for the Hotdata Rust SDK.
//!
//! Most callers never touch this module: they hand [`ClientBuilder::api_token`]
//! a long-lived `hd_` API token, it lands on
//! [`Configuration::bearer_access_token`](crate::apis::configuration::Configuration),
//! and every request sends it verbatim.
//!
//! A consumer that owns its own credential lifecycle needs more than a value
//! baked in at construction. The Hotdata CLI, for example, authenticates a user
//! through a PKCE browser login whose access token lives about five minutes and
//! must be refreshed mid-command — a multi-gigabyte `upload_file` whose finalize
//! call lands after the TTL, a long-running query, or a large parallel batch all
//! outlive the credential they started with. For those hosts, install a
//! [`BearerTokenProvider`] on
//! [`Configuration::token_provider`](crate::apis::configuration::Configuration)
//! and the SDK asks it for a bearer once per request instead:
//!
//! ```no_run
//! use hotdata::auth::{async_trait, BearerTokenError, BearerTokenProvider};
//! use hotdata::prelude::*;
//!
//! #[derive(Debug)]
//! struct MySession { /* refresh token, expiry, mutex, ... */ }
//!
//! #[async_trait]
//! impl BearerTokenProvider for MySession {
//!     async fn bearer_value(&self) -> Result<String, BearerTokenError> {
//!         // Refresh if needed, then hand back a currently valid access token.
//!         Ok("eyJ...".to_owned())
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = Client::builder().api_token("unused").build()?;
//! client.configuration_mut().token_provider = Some(std::sync::Arc::new(MySession {}));
//! # Ok(())
//! # }
//! ```
//!
//! This module is hand-written and regeneration-immune: OpenAPI Generator only
//! rewrites the files it emits, and `auth.rs` is additionally listed in
//! `.openapi-generator-ignore` as belt-and-suspenders.
//!
//! Nothing here exchanges one credential for another. The API-token -> JWT
//! exchange that earlier releases performed against `/v1/auth/jwt` is gone and
//! is not coming back; a provider is purely a hook for a host that already
//! knows how to produce a fresh bearer.
//!
//! [`ClientBuilder::api_token`]: crate::client::ClientBuilder::api_token

/// Re-exported so an implementor does not have to add its own `async-trait`
/// dependency (and cannot pick a version whose desugaring disagrees with ours).
/// Attach it to the `impl` block: `#[hotdata::auth::async_trait]`.
pub use async_trait::async_trait;

/// Raised when a [`BearerTokenProvider`] cannot produce a bearer token.
///
/// The variants cover what a provider that does its own HTTP typically hits.
/// A provider is free to use whichever fits; [`Malformed`](Self::Malformed)
/// doubles as the catch-all for a failure that isn't transport or status
/// shaped (an expired refresh token, a missing keyring entry, ...).
///
/// Marked `#[non_exhaustive]`: new failure modes may be added in future releases
/// without a breaking change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum BearerTokenError {
    /// Transport-level failure (connection refused, TLS error, timeout, ...).
    Transport(reqwest::Error),
    /// An upstream credential endpoint returned a non-success HTTP status.
    Status { status: u16, body: String },
    /// The credential could not be parsed, or is otherwise unusable.
    Malformed(String),
}

impl std::fmt::Display for BearerTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BearerTokenError::Transport(e) => {
                write!(f, "bearer token transport error: {e}")
            }
            BearerTokenError::Status { status, body } => {
                write!(f, "bearer token request failed: HTTP {status}: {body}")
            }
            BearerTokenError::Malformed(msg) => {
                write!(f, "malformed bearer token: {msg}")
            }
        }
    }
}

impl std::error::Error for BearerTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BearerTokenError::Transport(e) => Some(e),
            _ => None,
        }
    }
}

/// A pluggable async source of bearer tokens.
///
/// Installed on the generated `Configuration` as
/// `Option<Arc<dyn BearerTokenProvider>>`; the generated `resolve_bearer_token`
/// method calls [`bearer_value`](BearerTokenProvider::bearer_value) exactly once
/// per request, so a provider can hand back a freshly refreshed credential for
/// every call rather than one captured at `Client` construction.
///
/// Implementors are shared across concurrent requests behind an `Arc`, so
/// `bearer_value` takes `&self` and must be safe to call from several tasks at
/// once (typically a `tokio::sync::Mutex` around the refresh so concurrent
/// callers single-flight instead of stampeding). It is on the hot path of every
/// request, so the common case should be a cheap cache read.
#[async_trait::async_trait]
pub trait BearerTokenProvider: Send + Sync + std::fmt::Debug {
    /// Return the bearer token to put on the wire for the next request.
    async fn bearer_value(&self) -> Result<String, BearerTokenError>;
}
