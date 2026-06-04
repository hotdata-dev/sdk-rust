//! Transparent API-token -> JWT exchange for the Hotdata Rust SDK.
//!
//! Hotdata authenticates API requests with short-lived JWTs. Users still
//! configure the SDK with their long-lived `hd_` API token, but every request
//! should carry a fresh JWT instead. This module is the hand-written,
//! regeneration-immune piece that makes that happen behind the scenes:
//! [`TokenManager`] exchanges the API token for a JWT at
//! `POST {host}/v1/auth/jwt` and keeps it fresh, mirroring the CLI's `jwt.rs`
//! and the Python SDK's `hotdata/_auth.py` so the CLI and SDKs behave
//! identically.
//!
//! OpenAPI Generator only rewrites the files it emits, so this hand-added
//! module survives regeneration (precedent: the Python SDK's `_auth.py`). It is
//! additionally listed in `.openapi-generator-ignore` as belt-and-suspenders.
//!
//! Key behaviors:
//!
//! * **Pass-through** -- a credential that already looks like a JWT (`eyJ`
//!   prefix, matching the Gateway's own `^Bearer eyJ.*` detection) is returned
//!   unchanged and never exchanged. Every other (opaque) credential is treated
//!   as an API token and exchanged. (Hotdata API tokens are bare hex; the `hd_`
//!   prefix seen in docs is cosmetic and not enforced by the server, so we must
//!   not gate on it.)
//! * **Opt-out** -- if `HOTDATA_DISABLE_JWT_EXCHANGE` is set to an affirmative
//!   value (`1`/`true`/`yes`/`on`, trimmed + lowercased), the credential is
//!   always returned as-is (hard escape hatch for rollout / local dev). Other
//!   values (incl. `0`/`false`/empty) do NOT opt out.
//! * **In-memory cache only** -- no disk writes. The server already
//!   de-duplicates mints (keyed by `sha256(api_token)`), so per-process caching
//!   is sufficient.
//! * **Thread-safe single-flight** -- a [`tokio::sync::Mutex`] held across the
//!   mint request ensures concurrent first-requests perform exactly one mint.
//! * **Refresh, then re-mint** -- prefer the refresh token when available; on
//!   any refresh failure, drop it and re-mint from the held API token (always
//!   possible since the SDK holds it). Matches the CLI.
//! * **TLS/proxy reuse** -- the exchange reuses the SDK's configured
//!   `reqwest::Client` (cloned in by the [`crate::client::Client`] builder), so
//!   it honors the same TLS / proxy / timeout settings as every other request,
//!   with a bounded per-request timeout so a stalled token endpoint fails fast
//!   instead of hanging every call.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tokio::sync::Mutex;

/// `client_id` sent with every token-exchange request. Distinct from the CLI's
/// `hotdata-cli` and the Python SDK's `hotdata-python-sdk` so server-side logs
/// can attribute mints to the Rust SDK.
pub const CLIENT_ID: &str = "hotdata-rust-sdk";

/// Refresh early so callers don't race an expiring token (seconds).
pub const LEEWAY_SECS: u64 = 30;

/// Bounded timeout for the exchange request -- never let a stalled token
/// endpoint hang every request (seconds).
pub const TIMEOUT_SECS: u64 = 30;

/// Default access-token lifetime when the server omits `expires_in` (seconds).
const DEFAULT_EXPIRES_IN: u64 = 300;

/// Env var that disables exchange entirely. Hard escape hatch during the
/// rollout window and for local/dev setups. Only affirmative values opt out so
/// that `=0` / `=false` do NOT silently disable it.
const DISABLE_ENV: &str = "HOTDATA_DISABLE_JWT_EXCHANGE";

/// Affirmative opt-out values (compared trimmed + lowercased).
const DISABLE_VALUES: [&str; 4] = ["1", "true", "yes", "on"];

/// Raised when an API token cannot be exchanged for a JWT.
///
/// Surfacing the failure here (e.g. an `invalid_grant` from an expired/revoked
/// API token) keeps the cause clear instead of a confusing downstream 401.
///
/// Marked `#[non_exhaustive]`: new failure modes may be added in future releases
/// without a breaking change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum TokenExchangeError {
    /// Transport-level failure (connection refused, TLS error, timeout, ...).
    Transport(reqwest::Error),
    /// The token endpoint returned a non-success HTTP status.
    Status { status: u16, body: String },
    /// The response body could not be parsed, or lacked `access_token`.
    Malformed(String),
}

impl std::fmt::Display for TokenExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenExchangeError::Transport(e) => {
                write!(f, "token exchange transport error: {e}")
            }
            TokenExchangeError::Status { status, body } => {
                write!(f, "token exchange failed: HTTP {status}: {body}")
            }
            TokenExchangeError::Malformed(msg) => {
                write!(f, "malformed token response: {msg}")
            }
        }
    }
}

impl std::error::Error for TokenExchangeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TokenExchangeError::Transport(e) => Some(e),
            _ => None,
        }
    }
}

/// A pluggable async source of bearer tokens.
///
/// Installed on the generated `Configuration` as
/// `Option<Arc<dyn BearerTokenProvider>>`; the generated `resolve_bearer_token`
/// method calls [`bearer_value`](BearerTokenProvider::bearer_value) exactly once
/// per request. Implemented by [`TokenManager`]; users can supply their own.
#[async_trait::async_trait]
pub trait BearerTokenProvider: Send + Sync + std::fmt::Debug {
    /// Return the bearer token to put on the wire for the next request.
    async fn bearer_value(&self) -> Result<String, TokenExchangeError>;
}

/// Callback fired after a successful mint so a host (e.g. the CLI) can persist
/// the rotated tokens across process invocations.
///
/// Invoked with `(access_token, refresh_token, exp)` where `exp` is the
/// absolute unix-epoch expiry (seconds). `refresh_token` is `None` when the
/// server omitted one (rotation off; the prior refresh token is carried
/// forward and the callback is still given `None` to reflect the wire
/// response). The callback must return quickly and must not re-enter the
/// [`TokenManager`] (it runs while the single-flight lock is held).
pub type PersistCallback = Arc<dyn Fn(&str, Option<&str>, u64) + Send + Sync>;

/// Tuning for [`TokenManager::with_options`].
///
/// The default preserves the historical [`TokenManager::new`] behavior exactly:
/// `client_id = CLIENT_ID` (`hotdata-rust-sdk`), `token_path = /v1/auth/jwt`,
/// empty `base_path`, no seed, and no persistence callback. Hosts that mint
/// under a different attribution (e.g. the CLI's `hotdata-cli` at `/o/token/`)
/// or that need to seed an existing session and persist rotations override the
/// relevant fields.
#[non_exhaustive]
pub struct TokenManagerOptions {
    /// `client_id` form param sent with every mint. Defaults to [`CLIENT_ID`].
    pub client_id: String,
    /// Path appended to `base_path` for the mint endpoint. Defaults to
    /// `/v1/auth/jwt`.
    pub token_path: String,
    /// API base URL the mint POSTs to (e.g. `https://api.hotdata.dev`). The
    /// builder fills this from the resolved base path.
    pub base_path: String,
    /// Optional refresh token to seed the cache with, so the first request can
    /// take the refresh path instead of re-minting from the API token.
    pub seed_refresh: Option<String>,
    /// Optional `(jwt, exp)` to seed the cache with a known-valid JWT (absolute
    /// unix-epoch expiry in seconds), avoiding a mint on the first request.
    pub seed_jwt: Option<(String, u64)>,
    /// Optional callback fired after each successful mint so the host can
    /// persist the rotated tokens.
    pub on_persist: Option<PersistCallback>,
}

impl Default for TokenManagerOptions {
    fn default() -> Self {
        TokenManagerOptions {
            client_id: CLIENT_ID.to_string(),
            token_path: "/v1/auth/jwt".to_string(),
            base_path: String::new(),
            seed_refresh: None,
            seed_jwt: None,
            on_persist: None,
        }
    }
}

impl std::fmt::Debug for TokenManagerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenManagerOptions")
            .field("client_id", &self.client_id)
            .field("token_path", &self.token_path)
            .field("base_path", &self.base_path)
            .field("seed_refresh", &self.seed_refresh.as_ref().map(|_| "<redacted>"))
            .field("seed_jwt", &self.seed_jwt.as_ref().map(|(_, exp)| ("<redacted>", exp)))
            .field("on_persist", &self.on_persist.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

/// Token-exchange response (`POST /v1/auth/jwt`). Mirrors the CLI's
/// `TokenResponse` and the OAuth token-grant shape.
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

/// Cached JWT state. `exp` is an absolute unix timestamp (seconds).
#[derive(Debug, Default)]
struct TokenState {
    jwt: Option<String>,
    exp: u64,
    refresh: Option<String>,
}

/// Exchanges an API token for short-lived JWTs and keeps them fresh.
///
/// A credential that already looks like a JWT (`eyJ` prefix) is passed through
/// unchanged, as is any credential when `HOTDATA_DISABLE_JWT_EXCHANGE` is set
/// to an affirmative value; every other (opaque) API token is exchanged.
pub struct TokenManager {
    /// The user-supplied credential (API token, or a literal JWT to pass through).
    credential: String,
    /// The SDK's reqwest client, cloned in so TLS/proxy/pool settings are reused.
    client: reqwest::Client,
    /// API host the exchange POSTs to; read at mint time.
    base_path: String,
    /// Path appended to `base_path` for the mint endpoint (default `/v1/auth/jwt`).
    token_path: String,
    /// `client_id` form param sent with every mint (default [`CLIENT_ID`]).
    client_id: String,
    /// Optional callback fired after a successful mint so the host can persist
    /// rotated tokens across invocations.
    on_persist: Option<PersistCallback>,
    /// Cached JWT + refresh token, guarded for single-flight minting.
    state: Mutex<TokenState>,
}

impl std::fmt::Debug for TokenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenManager")
            .field("credential", &"<redacted>")
            .field("base_path", &self.base_path)
            .field("token_path", &self.token_path)
            .field("client_id", &self.client_id)
            .field("on_persist", &self.on_persist.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl TokenManager {
    /// Build a token manager.
    ///
    /// * `credential` -- the user's API token (or a literal `eyJ...` JWT).
    /// * `client` -- the SDK's configured reqwest client; cloned in so the
    ///   exchange reuses the same TLS/proxy/connection pool.
    /// * `base_path` -- the API base URL (e.g. `https://api.hotdata.dev`);
    ///   `/v1/auth/jwt` is appended at mint time.
    pub fn new(
        credential: impl Into<String>,
        client: reqwest::Client,
        base_path: impl Into<String>,
    ) -> Self {
        TokenManager::with_options(
            credential,
            client,
            TokenManagerOptions {
                base_path: base_path.into(),
                ..Default::default()
            },
        )
    }

    /// Build a token manager with explicit [`TokenManagerOptions`].
    ///
    /// Equivalent to [`TokenManager::new`] when `opts` is `Default::default()`
    /// with `base_path` set. Use this to override the mint attribution
    /// (`client_id`/`token_path`), seed an existing JWT or refresh token, or
    /// install an `on_persist` callback that survives rotation across process
    /// invocations.
    ///
    /// * `credential` -- the user's API token (or a literal `eyJ...` JWT).
    /// * `client` -- the SDK's configured reqwest client; cloned in so the
    ///   exchange reuses the same TLS/proxy/connection pool.
    /// * `opts` -- mint attribution, seed values, and persistence callback.
    pub fn with_options(
        credential: impl Into<String>,
        client: reqwest::Client,
        opts: TokenManagerOptions,
    ) -> Self {
        let mut state = TokenState::default();
        if let Some(refresh) = opts.seed_refresh {
            state.refresh = Some(refresh);
        }
        if let Some((jwt, exp)) = opts.seed_jwt {
            state.jwt = Some(jwt);
            state.exp = exp;
        }
        TokenManager {
            credential: credential.into(),
            client,
            base_path: opts.base_path,
            token_path: opts.token_path,
            client_id: opts.client_id,
            on_persist: opts.on_persist,
            state: Mutex::new(state),
        }
    }

    /// Whether the credential should be exchanged for a JWT.
    ///
    /// Opt-out wins outright: an affirmative `HOTDATA_DISABLE_JWT_EXCHANGE`
    /// means send the credential as-is, never touching the token endpoint. A
    /// credential that already starts with `eyJ` (a compact JWT) is likewise
    /// passed through. Everything else is an opaque API token to be exchanged.
    fn needs_exchange(&self) -> bool {
        if disable_exchange_env() {
            return false;
        }
        !self.credential.starts_with("eyJ")
    }

    /// Mint a JWT using the given form params, returning the parsed response.
    ///
    /// Always sends `client_id` and a bounded timeout, and reuses the shared
    /// reqwest client. Errors are returned as [`TokenExchangeError`]; the caller
    /// decides whether a given grant is best-effort (refresh) or hard
    /// (api_token).
    async fn mint(&self, grant: &[(&str, &str)]) -> Result<TokenResponse, TokenExchangeError> {
        let url = format!(
            "{}{}",
            self.base_path.trim_end_matches('/'),
            self.token_path
        );
        let mut params: Vec<(&str, &str)> = grant.to_vec();
        params.push(("client_id", &self.client_id));

        let resp = self
            .client
            .post(&url)
            .form(&params)
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .send()
            .await
            .map_err(TokenExchangeError::Transport)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            // Truncate to keep error messages bounded (mirrors python's [:200]).
            let body: String = body.chars().take(200).collect();
            return Err(TokenExchangeError::Status {
                status: status.as_u16(),
                body,
            });
        }

        let text = resp.text().await.map_err(TokenExchangeError::Transport)?;
        serde_json::from_str::<TokenResponse>(&text)
            .map_err(|e| TokenExchangeError::Malformed(e.to_string()))
    }

    /// Apply a successful mint response to the cached state.
    ///
    /// Carries the old refresh token forward when the server omits one (token
    /// rotation is off server-side, so the same refresh token is reused).
    fn apply(state: &mut TokenState, resp: TokenResponse) {
        let expires_in = resp.expires_in.unwrap_or(DEFAULT_EXPIRES_IN);
        state.jwt = Some(resp.access_token);
        state.exp = now_unix().saturating_add(expires_in);
        if let Some(refresh) = resp.refresh_token {
            state.refresh = Some(refresh);
        }
        // else: keep the existing refresh token (carry-forward).
    }

    /// Apply a successful mint and, if configured, fire the persistence
    /// callback with the freshly minted tokens.
    ///
    /// The callback receives the wire `refresh_token` (`None` when the server
    /// omitted one) and the absolute expiry written into the cache, so a host
    /// can persist exactly what landed. It runs while the single-flight lock is
    /// held, so it must be quick and non-reentrant.
    fn apply_and_persist(&self, state: &mut TokenState, resp: TokenResponse) {
        let wire_refresh = resp.refresh_token.clone();
        Self::apply(state, resp);
        if let Some(cb) = &self.on_persist {
            if let Some(jwt) = &state.jwt {
                cb(jwt, wire_refresh.as_deref(), state.exp);
            }
        }
    }
}

#[async_trait::async_trait]
impl BearerTokenProvider for TokenManager {
    async fn bearer_value(&self) -> Result<String, TokenExchangeError> {
        // Already a JWT (or opt-out) -> return unchanged, no network call,
        // no lock.
        if !self.needs_exchange() {
            return Ok(self.credential.clone());
        }

        // Single-flight: hold the lock across the mint so concurrent first
        // requests perform exactly one exchange. The bounded per-request
        // timeout caps how long a stalled endpoint can serialize callers.
        let mut state = self.state.lock().await;

        // Fast path: a still-valid cached JWT, no network call.
        if let Some(ref jwt) = state.jwt {
            if now_unix() + LEEWAY_SECS < state.exp {
                return Ok(jwt.clone());
            }
        }

        // Prefer the refresh token; best-effort -- on ANY failure, drop it and
        // fall through to re-mint from the held API token.
        if let Some(refresh) = state.refresh.clone() {
            match self
                .mint(&[("grant_type", "refresh_token"), ("refresh_token", &refresh)])
                .await
            {
                Ok(resp) => self.apply_and_persist(&mut state, resp),
                Err(_) => state.refresh = None,
            }
        }

        // Re-mint from the held API token if we still lack a fresh JWT. This is
        // the hard path: any failure is surfaced as a TokenExchangeError.
        let needs_mint = match state.jwt {
            Some(_) => now_unix() + LEEWAY_SECS >= state.exp,
            None => true,
        };
        if needs_mint {
            let resp = self
                .mint(&[("grant_type", "api_token"), ("api_token", &self.credential)])
                .await?;
            self.apply_and_persist(&mut state, resp);
        }

        // apply() always sets jwt on a successful mint; unwrap is safe because
        // the only paths here either set it or returned an error above.
        Ok(state.jwt.clone().expect("jwt set after successful mint"))
    }
}

/// Current unix time in whole seconds (saturating to 0 before the epoch).
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Whether `HOTDATA_DISABLE_JWT_EXCHANGE` is set to an affirmative value.
fn disable_exchange_env() -> bool {
    match std::env::var(DISABLE_ENV) {
        Ok(v) => DISABLE_VALUES.contains(&v.trim().to_ascii_lowercase().as_str()),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // --- Helpers -----------------------------------------------------------

    /// A TokenManager pointed at the given base URL. The reqwest client is a
    /// fresh default; tests that exercise the network point base_path at a
    /// wiremock server.
    fn manager(credential: &str, base_path: &str) -> TokenManager {
        TokenManager::new(credential, reqwest::Client::new(), base_path)
    }

    /// Serialize tests that mutate the shared process env var so they don't
    /// race. Each guard holds a process-global lock for its lifetime, sets the
    /// var on construction, and removes it on drop. The lock guarantees no two
    /// env-mutating tests run concurrently even though `cargo test` runs test
    /// functions on parallel threads.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
    }
    impl EnvGuard {
        fn set(value: &str) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            std::env::set_var(DISABLE_ENV, value);
            EnvGuard { _lock: lock }
        }
        fn unset() -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            std::env::remove_var(DISABLE_ENV);
            EnvGuard { _lock: lock }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(DISABLE_ENV);
        }
    }

    // --- Pass-through / opt-out detection (no network) ---------------------

    #[test]
    fn jwt_credential_is_passed_through() {
        let _g = EnvGuard::unset();
        // A compact JWT starts with the base64 of `{"` => "eyJ".
        let m = manager("eyJhbGciOiJIUzI1NiJ9.payload.sig", "http://127.0.0.1:1");
        assert!(
            !m.needs_exchange(),
            "literal JWTs must pass through unchanged"
        );
    }

    #[test]
    fn opaque_token_needs_exchange() {
        let _g = EnvGuard::unset();
        // hd_ prefix is cosmetic; bare hex tokens are also exchanged.
        assert!(manager("hd_deadbeef", "http://127.0.0.1:1").needs_exchange());
        assert!(manager("deadbeefcafef00d", "http://127.0.0.1:1").needs_exchange());
    }

    #[test]
    fn affirmative_optout_values_disable_exchange() {
        // Only 1/true/yes/on (trimmed + case-insensitive) opt out.
        for v in ["1", "true", "TRUE", "Yes", " on ", "On"] {
            let _g = EnvGuard::set(v);
            assert!(
                !manager("hd_opaque", "http://127.0.0.1:1").needs_exchange(),
                "value {v:?} should disable exchange"
            );
        }
    }

    #[test]
    fn non_affirmative_optout_values_keep_exchange() {
        for v in ["0", "false", "no", "off", "", " ", "maybe", "2"] {
            let _g = EnvGuard::set(v);
            assert!(
                manager("hd_opaque", "http://127.0.0.1:1").needs_exchange(),
                "value {v:?} must NOT disable exchange"
            );
        }
    }

    #[tokio::test]
    async fn optout_returns_opaque_credential_unchanged() {
        let _g = EnvGuard::set("1");
        // Even a non-JWT credential is returned as-is when opted out, and no
        // network call is made (base_path points at a dead port).
        let m = manager("hd_opaque", "http://127.0.0.1:1");
        assert_eq!(m.bearer_value().await.unwrap(), "hd_opaque");
    }

    #[tokio::test]
    async fn passthrough_returns_jwt_without_network() {
        let _g = EnvGuard::unset();
        let m = manager("eyJ.a.b", "http://127.0.0.1:1");
        assert_eq!(m.bearer_value().await.unwrap(), "eyJ.a.b");
    }

    // --- expiry / leeway logic (apply + fast path, no real network) --------

    #[test]
    fn apply_uses_default_expiry_when_missing() {
        let mut state = TokenState::default();
        let before = now_unix();
        TokenManager::apply(
            &mut state,
            TokenResponse {
                access_token: "jwt".into(),
                expires_in: None,
                refresh_token: None,
            },
        );
        assert_eq!(state.jwt.as_deref(), Some("jwt"));
        // exp should be ~now + DEFAULT_EXPIRES_IN (300s).
        let ttl = state.exp - before;
        assert!(
            (DEFAULT_EXPIRES_IN..=DEFAULT_EXPIRES_IN + 5).contains(&ttl),
            "ttl={ttl}"
        );
    }

    #[test]
    fn apply_carries_refresh_token_forward_when_omitted() {
        let mut state = TokenState {
            refresh: Some("old-refresh".into()),
            ..Default::default()
        };
        TokenManager::apply(
            &mut state,
            TokenResponse {
                access_token: "jwt".into(),
                expires_in: Some(300),
                refresh_token: None,
            },
        );
        assert_eq!(state.refresh.as_deref(), Some("old-refresh"));
    }

    #[test]
    fn apply_uses_rotated_refresh_token_when_present() {
        let mut state = TokenState {
            refresh: Some("old".into()),
            ..Default::default()
        };
        TokenManager::apply(
            &mut state,
            TokenResponse {
                access_token: "jwt".into(),
                expires_in: Some(300),
                refresh_token: Some("rotated".into()),
            },
        );
        assert_eq!(state.refresh.as_deref(), Some("rotated"));
    }

    #[tokio::test]
    async fn fast_path_returns_cached_jwt_without_network() {
        let _g = EnvGuard::unset();
        // Pre-seed a valid cached JWT well past the leeway window, then point
        // base_path at a dead port: if the fast path failed, the mint would
        // surface as an error.
        let m = manager("hd_opaque", "http://127.0.0.1:1");
        {
            let mut state = m.state.lock().await;
            state.jwt = Some("cached-jwt".into());
            state.exp = now_unix() + 600; // 10 min out, > LEEWAY
        }
        assert_eq!(m.bearer_value().await.unwrap(), "cached-jwt");
    }

    #[tokio::test]
    async fn cached_jwt_inside_leeway_is_not_used_directly() {
        let _g = EnvGuard::unset();
        // A JWT with only a few seconds of life left (inside LEEWAY) must NOT
        // be returned from the fast path; with a dead endpoint the re-mint
        // fails, proving the fast path was skipped.
        let m = manager("hd_opaque", "http://127.0.0.1:1");
        {
            let mut state = m.state.lock().await;
            state.jwt = Some("stale-jwt".into());
            state.exp = now_unix() + 5; // inside the 30s leeway
        }
        let err = m.bearer_value().await.unwrap_err();
        assert!(
            matches!(err, TokenExchangeError::Transport(_)),
            "expected a transport error from the re-mint attempt, got {err:?}"
        );
    }

    // --- mint / refresh-then-remint via wiremock ---------------------------

    #[tokio::test]
    async fn mints_from_api_token_when_cache_empty() {
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .and(body_string_contains("grant_type=api_token"))
            .and(body_string_contains("client_id=hotdata-rust-sdk"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "minted-jwt",
                "expires_in": 300,
                "refresh_token": "r1"
            })))
            .mount(&server)
            .await;

        let m = manager("hd_opaque", &server.uri());
        assert_eq!(m.bearer_value().await.unwrap(), "minted-jwt");
        // Cached: a second call returns the same JWT without re-minting (the
        // mock would still match, so assert the refresh token landed too).
        assert_eq!(m.bearer_value().await.unwrap(), "minted-jwt");
        assert_eq!(m.state.lock().await.refresh.as_deref(), Some("r1"));
    }

    #[tokio::test]
    async fn refresh_failure_falls_through_to_remint() {
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        // Refresh grant -> rejected (best-effort, drops the refresh token).
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(ResponseTemplate::new(400).set_body_string("invalid_grant"))
            .mount(&server)
            .await;
        // api_token grant -> succeeds (the hard re-mint path).
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .and(body_string_contains("grant_type=api_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "reminted-jwt",
                "expires_in": 300
            })))
            .mount(&server)
            .await;

        let m = manager("hd_opaque", &server.uri());
        // Seed an expired JWT + a refresh token so the refresh path is taken
        // first, fails, then the api_token re-mint runs.
        {
            let mut state = m.state.lock().await;
            state.jwt = Some("expired".into());
            state.exp = now_unix(); // already inside leeway / expired
            state.refresh = Some("dead-refresh".into());
        }
        assert_eq!(m.bearer_value().await.unwrap(), "reminted-jwt");
        // The dead refresh token was dropped (server omitted a new one).
        assert_eq!(m.state.lock().await.refresh, None);
    }

    #[tokio::test]
    async fn http_error_on_api_token_mint_is_surfaced() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .respond_with(ResponseTemplate::new(401).set_body_string("invalid api token"))
            .mount(&server)
            .await;

        let m = manager("revoked", &server.uri());
        let err = m.bearer_value().await.unwrap_err();
        match err {
            TokenExchangeError::Status { status, body } => {
                assert_eq!(status, 401);
                assert!(body.contains("invalid api token"), "body={body}");
            }
            other => panic!("expected Status error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn malformed_response_is_surfaced() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let m = manager("hd_opaque", &server.uri());
        assert!(matches!(
            m.bearer_value().await.unwrap_err(),
            TokenExchangeError::Malformed(_)
        ));
    }

    #[tokio::test]
    async fn single_flight_mints_once_under_concurrency() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;

        // Count how many times the endpoint is actually hit.
        struct Counter(Arc<AtomicUsize>);
        impl Respond for Counter {
            fn respond(&self, _: &Request) -> ResponseTemplate {
                self.0.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "access_token": "minted-jwt",
                    "expires_in": 300
                }))
            }
        }
        let hits = Arc::new(AtomicUsize::new(0));
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .respond_with(Counter(hits.clone()))
            .mount(&server)
            .await;

        let m = Arc::new(manager("hd_opaque", &server.uri()));
        // Fire many concurrent first-requests; single-flight must collapse them
        // into one mint.
        let mut handles = Vec::new();
        for _ in 0..16 {
            let m = m.clone();
            handles.push(tokio::spawn(async move { m.bearer_value().await }));
        }
        for h in handles {
            assert_eq!(h.await.unwrap().unwrap(), "minted-jwt");
        }
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "single-flight must mint once"
        );
    }

    #[tokio::test]
    async fn trailing_slash_in_base_path_is_normalized() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt")) // not "//v1/auth/jwt"
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "ok",
                "expires_in": 300
            })))
            .mount(&server)
            .await;

        let base = format!("{}/", server.uri());
        let m = manager("hd_opaque", &base);
        assert_eq!(m.bearer_value().await.unwrap(), "ok");
    }

    // --- TokenManagerOptions: defaults, attribution, seed, persist ---------

    #[test]
    fn new_matches_default_options() {
        // new() must be byte-for-byte equivalent to with_options(.., Default)
        // with base_path set: same client_id, token_path, no seed, no persist.
        let m = TokenManager::new("hd_opaque", reqwest::Client::new(), "https://api.example.dev");
        assert_eq!(m.client_id, CLIENT_ID);
        assert_eq!(m.token_path, "/v1/auth/jwt");
        assert_eq!(m.base_path, "https://api.example.dev");
        assert!(m.on_persist.is_none());
    }

    #[test]
    fn options_default_preserves_legacy_attribution() {
        let opts = TokenManagerOptions::default();
        assert_eq!(opts.client_id, CLIENT_ID);
        assert_eq!(opts.token_path, "/v1/auth/jwt");
        assert!(opts.base_path.is_empty());
        assert!(opts.seed_refresh.is_none());
        assert!(opts.seed_jwt.is_none());
        assert!(opts.on_persist.is_none());
    }

    #[tokio::test]
    async fn seed_jwt_is_served_without_minting() {
        let _g = EnvGuard::unset();
        // A seeded, still-valid JWT must be returned from the fast path; the
        // dead port proves no mint happened.
        let m = TokenManager::with_options(
            "hd_opaque",
            reqwest::Client::new(),
            TokenManagerOptions {
                base_path: "http://127.0.0.1:1".into(),
                seed_jwt: Some(("seeded-jwt".into(), now_unix() + 600)),
                ..Default::default()
            },
        );
        assert_eq!(m.bearer_value().await.unwrap(), "seeded-jwt");
    }

    #[tokio::test]
    async fn seed_refresh_drives_refresh_grant_first() {
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        // Only the refresh grant is mounted; if the manager re-minted from the
        // api_token instead, the request would 404 and surface an error.
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .and(body_string_contains("grant_type=refresh_token"))
            .and(body_string_contains("refresh_token=seeded-refresh"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "from-seeded-refresh",
                "expires_in": 300
            })))
            .mount(&server)
            .await;

        let m = TokenManager::with_options(
            "hd_opaque",
            reqwest::Client::new(),
            TokenManagerOptions {
                base_path: server.uri(),
                seed_refresh: Some("seeded-refresh".into()),
                ..Default::default()
            },
        );
        assert_eq!(m.bearer_value().await.unwrap(), "from-seeded-refresh");
    }

    #[tokio::test]
    async fn configurable_client_id_and_token_path_hit_the_wire() {
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        // Mint must POST to the configured path with the configured client_id
        // (the CLI's `hotdata-cli` at `/o/token/`), NOT the SDK defaults.
        Mock::given(method("POST"))
            .and(path("/o/token/"))
            .and(body_string_contains("client_id=hotdata-cli"))
            .and(body_string_contains("grant_type=api_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "cli-minted",
                "expires_in": 300
            })))
            .mount(&server)
            .await;

        let m = TokenManager::with_options(
            "hd_opaque",
            reqwest::Client::new(),
            TokenManagerOptions {
                base_path: server.uri(),
                client_id: "hotdata-cli".into(),
                token_path: "/o/token/".into(),
                ..Default::default()
            },
        );
        assert_eq!(m.bearer_value().await.unwrap(), "cli-minted");
    }

    #[tokio::test]
    async fn on_persist_fires_on_mint_with_rotated_tokens() {
        use std::sync::Mutex as StdMutex;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "minted-jwt",
                "expires_in": 300,
                "refresh_token": "rotated-refresh"
            })))
            .mount(&server)
            .await;

        // Capture what the callback was handed.
        let captured: Arc<StdMutex<Option<(String, Option<String>, u64)>>> =
            Arc::new(StdMutex::new(None));
        let sink = captured.clone();
        let cb: PersistCallback = Arc::new(move |jwt: &str, refresh: Option<&str>, exp: u64| {
            *sink.lock().unwrap() = Some((jwt.to_string(), refresh.map(str::to_string), exp));
        });

        let before = now_unix();
        let m = TokenManager::with_options(
            "hd_opaque",
            reqwest::Client::new(),
            TokenManagerOptions {
                base_path: server.uri(),
                on_persist: Some(cb),
                ..Default::default()
            },
        );
        assert_eq!(m.bearer_value().await.unwrap(), "minted-jwt");

        let got = captured.lock().unwrap().clone().expect("on_persist must fire");
        assert_eq!(got.0, "minted-jwt");
        assert_eq!(got.1.as_deref(), Some("rotated-refresh"));
        assert!(got.2 >= before + 300, "exp should be ~now+expires_in");
    }

    #[tokio::test]
    async fn on_persist_reports_none_refresh_when_server_omits_it() {
        use std::sync::Mutex as StdMutex;
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = EnvGuard::unset();
        let server = MockServer::start().await;
        // Refresh grant succeeds but omits a new refresh token (rotation off).
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "refreshed-jwt",
                "expires_in": 300
            })))
            .mount(&server)
            .await;

        let captured: Arc<StdMutex<Option<Option<String>>>> = Arc::new(StdMutex::new(None));
        let sink = captured.clone();
        let cb: PersistCallback = Arc::new(move |_jwt: &str, refresh: Option<&str>, _exp: u64| {
            *sink.lock().unwrap() = Some(refresh.map(str::to_string));
        });

        let m = TokenManager::with_options(
            "hd_opaque",
            reqwest::Client::new(),
            TokenManagerOptions {
                base_path: server.uri(),
                seed_refresh: Some("seeded".into()),
                on_persist: Some(cb),
                ..Default::default()
            },
        );
        assert_eq!(m.bearer_value().await.unwrap(), "refreshed-jwt");
        // The wire response omitted refresh_token -> callback sees None, even
        // though the cache carries the prior refresh token forward.
        assert_eq!(captured.lock().unwrap().clone(), Some(None));
        assert_eq!(m.state.lock().await.refresh.as_deref(), Some("seeded"));
    }

    #[tokio::test]
    async fn on_persist_does_not_fire_for_seeded_jwt_fast_path() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let _g = EnvGuard::unset();
        let calls = Arc::new(AtomicUsize::new(0));
        let sink = calls.clone();
        let cb: PersistCallback = Arc::new(move |_: &str, _: Option<&str>, _: u64| {
            sink.fetch_add(1, Ordering::SeqCst);
        });

        let m = TokenManager::with_options(
            "hd_opaque",
            reqwest::Client::new(),
            TokenManagerOptions {
                base_path: "http://127.0.0.1:1".into(),
                seed_jwt: Some(("seeded".into(), now_unix() + 600)),
                on_persist: Some(cb),
                ..Default::default()
            },
        );
        assert_eq!(m.bearer_value().await.unwrap(), "seeded");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "no mint occurred, so on_persist must not fire"
        );
    }
}
