//! Ergonomic, hand-written entry point for the HotData SDK.
//!
//! This module is regeneration-immune: it is protected by `.openapi-generator-ignore`
//! and is never emitted by the OpenAPI generator. It wraps the generated
//! [`Configuration`](crate::apis::configuration::Configuration) and the
//! hand-written [`TokenManager`](crate::auth::TokenManager) to provide a flat,
//! low-ceremony surface that mirrors the Python SDK's top-level `hotdata` API.
//!
//! # Example
//!
//! ```no_run
//! use hotdata::prelude::*;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_token("hd_live_...")          // opaque token, JWT exchange is transparent
//!     .workspace_id("ws_public_id")      // sets the X-Workspace-Id header
//!     .build()?;
//!
//! let run = client.query(QueryRequest::new("select 1".into())).await?;
//! # let _ = run;
//! # Ok(())
//! # }
//! ```

use std::env;

use crate::apis::configuration::{ApiKey, Configuration};
use crate::apis::{self, Error};
use crate::auth::TokenManager;
use crate::models;

/// Default API host. Matches the generated `Configuration::default()` base
/// path and the OpenAPI spec server. The JWT exchange endpoint
/// (`/v1/auth/jwt`) lives on this API host, so the ergonomic `Client` sets
/// `base_path` explicitly to keep token exchange routed correctly even if a
/// caller starts from a `Configuration` with a different host.
pub const DEFAULT_BASE_URL: &str = "https://api.hotdata.dev";

/// Header name used to scope requests to a workspace. Inserted into
/// `Configuration::api_keys` so the generated apiKey-auth blocks emit it.
pub const WORKSPACE_ID_HEADER: &str = "X-Workspace-Id";

/// Header name used to scope requests to a session (optional).
pub const SESSION_ID_HEADER: &str = "X-Session-Id";

/// Environment variable holding the API token used for transparent JWT exchange.
/// Mirrors the Python SDK's `HOTDATA_API_KEY`.
pub const ENV_API_KEY: &str = "HOTDATA_API_KEY";

/// Environment variable holding the workspace public id.
pub const ENV_WORKSPACE_ID: &str = "HOTDATA_WORKSPACE_ID";

/// Environment variable overriding the base URL. Used by integration tests to
/// point at a non-production host (mirrors the Python SDK's test override).
pub const ENV_API_URL: &str = "HOTDATA_API_URL";

/// Test-only override for the base URL (takes precedence over `HOTDATA_API_URL`).
/// Mirrors the Python SDK's `HOTDATA_SDK_TEST_API_URL`.
pub const ENV_TEST_API_URL: &str = "HOTDATA_SDK_TEST_API_URL";

/// Errors that can occur while constructing a [`Client`].
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum ClientError {
    /// No API token was supplied (neither via the builder nor the environment).
    MissingApiToken,
    /// No workspace id was supplied (neither via the builder nor the environment).
    MissingWorkspaceId,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::MissingApiToken => write!(
                f,
                "no API token supplied; set it via ClientBuilder::api_token or the {ENV_API_KEY} environment variable"
            ),
            ClientError::MissingWorkspaceId => write!(
                f,
                "no workspace id supplied; set it via ClientBuilder::workspace_id or the {ENV_WORKSPACE_ID} environment variable"
            ),
        }
    }
}

impl std::error::Error for ClientError {}

/// Builder for [`Client`].
///
/// Use [`Client::builder`] to obtain one. All fields are optional at the type
/// level; missing required fields fall back to environment variables and then
/// produce a [`ClientError`] at [`build`](ClientBuilder::build) time.
#[derive(Debug, Default, Clone)]
pub struct ClientBuilder {
    api_token: Option<String>,
    workspace_id: Option<String>,
    session_id: Option<String>,
    base_url: Option<String>,
    user_agent: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// Set the opaque API token (e.g. `hd_live_...`). The token is exchanged for
    /// a short-lived JWT transparently on the first authenticated request; an
    /// already-minted JWT (`eyJ...`) is passed through unchanged.
    pub fn api_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = Some(token.into());
        self
    }

    /// Set the workspace public id. Installed as the `X-Workspace-Id` header on
    /// every request.
    pub fn workspace_id(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    /// Set an optional session id. Installed as the `X-Session-Id` header when
    /// present.
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Override the base URL. Defaults to [`DEFAULT_BASE_URL`], or the
    /// `HOTDATA_SDK_TEST_API_URL` / `HOTDATA_API_URL` environment variables when
    /// set (in that order of precedence).
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Override the `User-Agent` header. Defaults to `hotdata-rust/<crate version>`.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Supply a pre-configured `reqwest::Client` (custom TLS, proxy, timeouts,
    /// connection pool, etc.). The same client is reused for both API calls and
    /// the out-of-band JWT exchange so transport settings are shared.
    pub fn reqwest_client(mut self, client: reqwest::Client) -> Self {
        self.reqwest_client = Some(client);
        self
    }

    /// Resolve the effective base URL, honoring the explicit builder value first,
    /// then the test override, then the generic override, then the default.
    fn resolve_base_url(&self) -> String {
        if let Some(ref url) = self.base_url {
            return url.clone();
        }
        if let Some(url) = non_empty_env(ENV_TEST_API_URL) {
            return url;
        }
        if let Some(url) = non_empty_env(ENV_API_URL) {
            return url;
        }
        DEFAULT_BASE_URL.to_owned()
    }

    /// Construct the [`Client`].
    ///
    /// Required values (api token, workspace id) fall back to the
    /// [`ENV_API_KEY`] / [`ENV_WORKSPACE_ID`] environment variables. A missing
    /// required value produces a [`ClientError`].
    pub fn build(self) -> Result<Client, ClientError> {
        let api_token = self
            .api_token
            .clone()
            .or_else(|| non_empty_env(ENV_API_KEY))
            .ok_or(ClientError::MissingApiToken)?;

        let workspace_id = self
            .workspace_id
            .clone()
            .or_else(|| non_empty_env(ENV_WORKSPACE_ID))
            .ok_or(ClientError::MissingWorkspaceId)?;

        let base_path = self.resolve_base_url();
        let http_client = self.reqwest_client.clone().unwrap_or_default();
        let user_agent = self
            .user_agent
            .clone()
            .unwrap_or_else(|| format!("hotdata-rust/{}", env!("CARGO_PKG_VERSION")));

        let mut configuration = Configuration {
            base_path: base_path.clone(),
            user_agent: Some(user_agent),
            client: http_client.clone(),
            ..Configuration::default()
        };

        // Scope every request to the workspace (and optionally the session) via
        // the generated apiKey-header auth blocks.
        configuration.api_keys.insert(
            WORKSPACE_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: workspace_id,
            },
        );
        if let Some(session_id) = self
            .session_id
            .clone()
            .or_else(|| non_empty_env("HOTDATA_SESSION_ID"))
        {
            configuration.api_keys.insert(
                SESSION_ID_HEADER.to_owned(),
                ApiKey {
                    prefix: None,
                    key: session_id,
                },
            );
        }

        // Install the transparent api_token -> JWT exchange. The TokenManager
        // reuses the same reqwest client (so TLS/proxy/timeout settings are
        // shared) and the resolved base path (so the JWT mint targets the API
        // host, honoring any test override).
        let token_manager = TokenManager::new(api_token, http_client, base_path);
        configuration.token_provider = Some(std::sync::Arc::new(token_manager));

        Ok(Client { configuration })
    }
}

/// Flat, ergonomic HotData client.
///
/// Wraps a generated [`Configuration`] with transparent JWT exchange and a
/// workspace-scoped header. Common operations are exposed as thin async
/// pass-throughs; for the full generated surface use
/// [`Client::configuration`] with any `hotdata::apis::*_api::*` free function.
#[derive(Debug, Clone)]
pub struct Client {
    configuration: Configuration,
}

impl Client {
    /// Start building a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Construct directly from a fully-formed [`Configuration`]. Power users who
    /// have already wired up authentication, a workspace header, and a base path
    /// can use this to get the ergonomic pass-throughs without the builder.
    pub fn from_configuration(configuration: Configuration) -> Self {
        Client { configuration }
    }

    /// Borrow the underlying [`Configuration`] so any generated API free
    /// function can be called directly, e.g.
    /// `hotdata::apis::datasets_api::list_datasets(client.configuration(), ..)`.
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    /// Mutably borrow the underlying [`Configuration`] for advanced tweaks after
    /// construction.
    pub fn configuration_mut(&mut self) -> &mut Configuration {
        &mut self.configuration
    }

    // --- Common pass-throughs -------------------------------------------------

    /// Execute a SQL query. Thin wrapper over
    /// [`apis::query_api::query`](crate::apis::query_api::query).
    pub async fn query(
        &self,
        request: models::QueryRequest,
    ) -> Result<models::QueryResponse, Error<apis::query_api::QueryError>> {
        apis::query_api::query(&self.configuration, request, None).await
    }

    /// List recent query runs.
    pub async fn list_query_runs(
        &self,
        limit: Option<i32>,
        cursor: Option<&str>,
    ) -> Result<models::ListQueryRunsResponse, Error<apis::query_runs_api::ListQueryRunsError>>
    {
        apis::query_runs_api::list_query_runs(&self.configuration, limit, cursor, None, None).await
    }

    /// List persisted results.
    pub async fn list_results(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<models::ListResultsResponse, Error<apis::results_api::ListResultsError>> {
        apis::results_api::list_results(&self.configuration, limit, offset).await
    }

    /// Fetch a persisted result by id (JSON form). For Arrow IPC decoding use
    /// [`Client::get_result_arrow`] (requires the `arrow` feature).
    pub async fn get_result(
        &self,
        id: &str,
    ) -> Result<models::GetResultResponse, Error<apis::results_api::GetResultError>> {
        apis::results_api::get_result(&self.configuration, id, None, None, None).await
    }

    /// List workspaces visible to the authenticated principal.
    pub async fn list_workspaces(
        &self,
        organization_public_id: Option<&str>,
    ) -> Result<models::ListWorkspacesResponse, Error<apis::workspaces_api::ListWorkspacesError>>
    {
        apis::workspaces_api::list_workspaces(&self.configuration, organization_public_id).await
    }

    // --- Arrow helpers (feature-gated) ---------------------------------------

    /// Fetch a result as Arrow IPC and decode it into record batches.
    ///
    /// Requires the `arrow` cargo feature.
    #[cfg(feature = "arrow")]
    pub async fn get_result_arrow(
        &self,
        id: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<crate::arrow::ArrowResult, crate::arrow::ArrowError> {
        crate::arrow::get_result_arrow(&self.configuration, id, offset, limit).await
    }

    /// Fetch a result as Arrow IPC and lazily iterate over its record batches.
    ///
    /// Requires the `arrow` cargo feature.
    #[cfg(feature = "arrow")]
    pub async fn stream_result_arrow(
        &self,
        id: &str,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<crate::arrow::ArrowBatchStream, crate::arrow::ArrowError> {
        crate::arrow::stream_result_arrow(&self.configuration, id, offset, limit).await
    }

    // --- Resource handles -----------------------------------------------------
    //
    // Grouped, ergonomic accessors over the generated `apis::*_api` free
    // functions so callers write `client.datasets().create(req)` instead of
    // `datasets_api::create_dataset(client.configuration(), req, ..)`. Each
    // handle borrows the `Configuration`; see `crate::resources`.

    /// Datasets resource handle.
    pub fn datasets(&self) -> crate::resources::DatasetsApi<'_> {
        crate::resources::DatasetsApi::new(&self.configuration)
    }

    /// Connections resource handle.
    pub fn connections(&self) -> crate::resources::ConnectionsApi<'_> {
        crate::resources::ConnectionsApi::new(&self.configuration)
    }

    /// Connection-types resource handle.
    pub fn connection_types(&self) -> crate::resources::ConnectionTypesApi<'_> {
        crate::resources::ConnectionTypesApi::new(&self.configuration)
    }

    /// Database-context resource handle.
    pub fn database_context(&self) -> crate::resources::DatabaseContextApi<'_> {
        crate::resources::DatabaseContextApi::new(&self.configuration)
    }

    /// Databases resource handle.
    pub fn databases(&self) -> crate::resources::DatabasesApi<'_> {
        crate::resources::DatabasesApi::new(&self.configuration)
    }

    /// Embedding-providers resource handle.
    pub fn embedding_providers(&self) -> crate::resources::EmbeddingProvidersApi<'_> {
        crate::resources::EmbeddingProvidersApi::new(&self.configuration)
    }

    /// Indexes resource handle.
    pub fn indexes(&self) -> crate::resources::IndexesApi<'_> {
        crate::resources::IndexesApi::new(&self.configuration)
    }

    /// Information-schema resource handle.
    pub fn information_schema(&self) -> crate::resources::InformationSchemaApi<'_> {
        crate::resources::InformationSchemaApi::new(&self.configuration)
    }

    /// Jobs resource handle.
    pub fn jobs(&self) -> crate::resources::JobsApi<'_> {
        crate::resources::JobsApi::new(&self.configuration)
    }

    /// Query resource handle (the flat [`Client::query`] shortcut covers the
    /// common case).
    pub fn queries(&self) -> crate::resources::QueryApi<'_> {
        crate::resources::QueryApi::new(&self.configuration)
    }

    /// Query-runs resource handle.
    pub fn query_runs(&self) -> crate::resources::QueryRunsApi<'_> {
        crate::resources::QueryRunsApi::new(&self.configuration)
    }

    /// Results resource handle.
    pub fn results(&self) -> crate::resources::ResultsApi<'_> {
        crate::resources::ResultsApi::new(&self.configuration)
    }

    /// Dataset-refresh resource handle.
    pub fn refresh(&self) -> crate::resources::RefreshApi<'_> {
        crate::resources::RefreshApi::new(&self.configuration)
    }

    /// Sandboxes resource handle.
    pub fn sandboxes(&self) -> crate::resources::SandboxesApi<'_> {
        crate::resources::SandboxesApi::new(&self.configuration)
    }

    /// Saved-queries resource handle.
    pub fn saved_queries(&self) -> crate::resources::SavedQueriesApi<'_> {
        crate::resources::SavedQueriesApi::new(&self.configuration)
    }

    /// Secrets resource handle.
    pub fn secrets(&self) -> crate::resources::SecretsApi<'_> {
        crate::resources::SecretsApi::new(&self.configuration)
    }

    /// Uploads resource handle.
    pub fn uploads(&self) -> crate::resources::UploadsApi<'_> {
        crate::resources::UploadsApi::new(&self.configuration)
    }

    /// Workspaces resource handle (the flat [`Client::list_workspaces`]
    /// shortcut covers the common case).
    pub fn workspaces(&self) -> crate::resources::WorkspacesApi<'_> {
        crate::resources::WorkspacesApi::new(&self.configuration)
    }

    // --- Query convenience helpers -------------------------------------------

    /// Poll a persisted result until it reaches `ready`.
    ///
    /// [`query`](Client::query) returns rows inline, but persistence to storage
    /// (required to re-fetch a result later, e.g. as Arrow) completes
    /// asynchronously. This polls [`get_result`](Client::get_result) on
    /// `result_id` until its `status` is `ready`, then returns that response.
    /// A `failed` status returns [`AwaitResultError::Failed`]; exceeding
    /// `poll.timeout` returns [`AwaitResultError::Timeout`]. Use
    /// `PollConfig::default()` for sensible defaults (120s timeout, 1s interval).
    pub async fn await_result(
        &self,
        result_id: &str,
        poll: PollConfig,
    ) -> Result<models::GetResultResponse, AwaitResultError> {
        let deadline = std::time::Instant::now() + poll.timeout;
        loop {
            let result = self
                .get_result(result_id)
                .await
                .map_err(AwaitResultError::Api)?;
            match result.status.as_str() {
                "ready" => return Ok(result),
                "failed" => {
                    return Err(AwaitResultError::Failed {
                        result_id: result_id.to_owned(),
                        error_message: result.error_message.flatten(),
                    })
                }
                _ => {}
            }
            if std::time::Instant::now() >= deadline {
                return Err(AwaitResultError::Timeout {
                    result_id: result_id.to_owned(),
                    last_status: result.status,
                    waited: poll.timeout,
                });
            }
            tokio::time::sleep(poll.interval).await;
        }
    }

    /// Submit a query and fetch its persisted result as Arrow IPC in one call.
    ///
    /// Runs the query, waits for the result to reach `ready` (see
    /// [`await_result`](Client::await_result)), then decodes it with
    /// [`get_result_arrow`](Client::get_result_arrow). Returns
    /// [`QueryToArrowError::NoResultId`] when the query could not be persisted
    /// (no `result_id`, e.g. catalog registration failed). Requires the `arrow`
    /// cargo feature.
    #[cfg(feature = "arrow")]
    pub async fn query_to_arrow(
        &self,
        request: models::QueryRequest,
        poll: PollConfig,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<crate::arrow::ArrowResult, QueryToArrowError> {
        let submitted = self
            .query(request)
            .await
            .map_err(QueryToArrowError::Query)?;
        let result_id =
            submitted
                .result_id
                .flatten()
                .ok_or_else(|| QueryToArrowError::NoResultId {
                    warning: submitted.warning.flatten(),
                })?;
        self.await_result(&result_id, poll)
            .await
            .map_err(QueryToArrowError::Await)?;
        self.get_result_arrow(&result_id, offset, limit)
            .await
            .map_err(QueryToArrowError::Arrow)
    }
}

/// How long to wait, and how often to poll, when awaiting a result.
///
/// Defaults to a 120-second timeout polled every second. Use
/// [`PollConfig::default`] for the common case, or construct one directly to
/// tune it.
#[derive(Debug, Clone, Copy)]
pub struct PollConfig {
    /// Maximum total time to wait for the result to become `ready`.
    pub timeout: std::time::Duration,
    /// Delay between successive polls.
    pub interval: std::time::Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        PollConfig {
            timeout: std::time::Duration::from_secs(120),
            interval: std::time::Duration::from_secs(1),
        }
    }
}

/// Error returned by [`Client::await_result`].
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum AwaitResultError {
    /// The underlying `get_result` call failed.
    Api(Error<apis::results_api::GetResultError>),
    /// The result reached `failed` status.
    Failed {
        /// The result id that failed.
        result_id: String,
        /// The server-provided failure message, when present.
        error_message: Option<String>,
    },
    /// The result did not become `ready` before the poll timeout elapsed.
    Timeout {
        /// The result id being awaited.
        result_id: String,
        /// The last status observed before timing out.
        last_status: String,
        /// How long was waited before giving up.
        waited: std::time::Duration,
    },
}

impl std::fmt::Display for AwaitResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwaitResultError::Api(e) => write!(f, "failed to fetch result: {e}"),
            AwaitResultError::Failed {
                result_id,
                error_message,
            } => write!(
                f,
                "result {result_id} failed: {}",
                error_message.as_deref().unwrap_or("no error message")
            ),
            AwaitResultError::Timeout {
                result_id,
                last_status,
                waited,
            } => write!(
                f,
                "result {result_id} did not become ready within {waited:?} (last status: {last_status})"
            ),
        }
    }
}

impl std::error::Error for AwaitResultError {}

/// Error returned by [`Client::query_to_arrow`].
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[cfg(feature = "arrow")]
#[derive(Debug)]
#[non_exhaustive]
pub enum QueryToArrowError {
    /// The query submission failed.
    Query(Error<apis::query_api::QueryError>),
    /// The query ran but could not be persisted, so it has no `result_id` to
    /// re-fetch as Arrow. `warning` carries the server's explanation, if any.
    NoResultId {
        /// The server-provided warning explaining why persistence was skipped.
        warning: Option<String>,
    },
    /// Awaiting the result to become ready failed (or timed out).
    Await(AwaitResultError),
    /// Decoding the ready result as Arrow IPC failed.
    Arrow(crate::arrow::ArrowError),
}

#[cfg(feature = "arrow")]
impl std::fmt::Display for QueryToArrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryToArrowError::Query(e) => write!(f, "query failed: {e}"),
            QueryToArrowError::NoResultId { warning } => write!(
                f,
                "query result was not persisted, cannot fetch as Arrow: {}",
                warning.as_deref().unwrap_or("no result_id returned")
            ),
            QueryToArrowError::Await(e) => write!(f, "{e}"),
            QueryToArrowError::Arrow(e) => write!(f, "arrow decode failed: {e}"),
        }
    }
}

#[cfg(feature = "arrow")]
impl std::error::Error for QueryToArrowError {}

/// Read an environment variable, treating empty/whitespace-only values as absent.
fn non_empty_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialize env-mutating tests so they don't race each other. `std::env`
    /// is process-global, so concurrent test threads would otherwise interfere.
    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn clear_env() {
        for key in [
            ENV_API_KEY,
            ENV_WORKSPACE_ID,
            ENV_API_URL,
            ENV_TEST_API_URL,
            "HOTDATA_SESSION_ID",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn builder_explicit_values_win() {
        let _g = env_guard();
        clear_env();

        let client = Client::builder()
            .api_token("hd_explicit")
            .workspace_id("ws_explicit")
            .base_url("https://example.test")
            .build()
            .expect("build should succeed with explicit values");

        let config = client.configuration();
        assert_eq!(config.base_path, "https://example.test");
        assert_eq!(
            config
                .api_keys
                .get(WORKSPACE_ID_HEADER)
                .map(|k| k.key.as_str()),
            Some("ws_explicit")
        );
        assert!(
            config.token_provider.is_some(),
            "a token provider must be installed"
        );
    }

    #[test]
    fn builder_falls_back_to_env() {
        let _g = env_guard();
        clear_env();
        env::set_var(ENV_API_KEY, "hd_from_env");
        env::set_var(ENV_WORKSPACE_ID, "ws_from_env");

        let client = Client::builder().build().expect("env fallback should work");
        let config = client.configuration();

        assert_eq!(
            config
                .api_keys
                .get(WORKSPACE_ID_HEADER)
                .map(|k| k.key.as_str()),
            Some("ws_from_env")
        );
        // Default base URL when nothing overrides it.
        assert_eq!(config.base_path, DEFAULT_BASE_URL);

        clear_env();
    }

    #[test]
    fn explicit_api_token_beats_env() {
        let _g = env_guard();
        clear_env();
        env::set_var(ENV_API_KEY, "hd_from_env");
        env::set_var(ENV_WORKSPACE_ID, "ws_from_env");

        // Explicit workspace should override the env one; token still resolves.
        let client = Client::builder()
            .api_token("hd_explicit")
            .workspace_id("ws_explicit")
            .build()
            .expect("build should succeed");

        assert_eq!(
            client
                .configuration()
                .api_keys
                .get(WORKSPACE_ID_HEADER)
                .map(|k| k.key.as_str()),
            Some("ws_explicit")
        );

        clear_env();
    }

    #[test]
    fn missing_token_errors() {
        let _g = env_guard();
        clear_env();

        let err = Client::builder()
            .workspace_id("ws_only")
            .build()
            .expect_err("missing token must error");
        assert!(matches!(err, ClientError::MissingApiToken));
    }

    #[test]
    fn missing_workspace_errors() {
        let _g = env_guard();
        clear_env();

        let err = Client::builder()
            .api_token("hd_only")
            .build()
            .expect_err("missing workspace must error");
        assert!(matches!(err, ClientError::MissingWorkspaceId));
    }

    #[test]
    fn test_url_override_precedence() {
        let _g = env_guard();
        clear_env();
        env::set_var(ENV_API_KEY, "hd_x");
        env::set_var(ENV_WORKSPACE_ID, "ws_x");
        // Both set; the test override must win.
        env::set_var(ENV_API_URL, "https://generic.test");
        env::set_var(ENV_TEST_API_URL, "https://test-override.test");

        let client = Client::builder().build().expect("build ok");
        assert_eq!(
            client.configuration().base_path,
            "https://test-override.test"
        );

        // With only the generic override present it is used.
        env::remove_var(ENV_TEST_API_URL);
        let client = Client::builder().build().expect("build ok");
        assert_eq!(client.configuration().base_path, "https://generic.test");

        // An explicit builder base_url beats both env vars.
        let client = Client::builder()
            .base_url("https://explicit.test")
            .build()
            .expect("build ok");
        assert_eq!(client.configuration().base_path, "https://explicit.test");

        clear_env();
    }

    #[test]
    fn empty_env_treated_as_absent() {
        let _g = env_guard();
        clear_env();
        env::set_var(ENV_API_KEY, "   ");
        env::set_var(ENV_WORKSPACE_ID, "ws_present");

        let err = Client::builder()
            .build()
            .expect_err("whitespace-only token must be treated as absent");
        assert!(matches!(err, ClientError::MissingApiToken));

        clear_env();
    }

    #[test]
    fn session_id_installed_when_set() {
        let _g = env_guard();
        clear_env();

        let client = Client::builder()
            .api_token("hd_x")
            .workspace_id("ws_x")
            .session_id("sess_123")
            .build()
            .expect("build ok");

        assert_eq!(
            client
                .configuration()
                .api_keys
                .get(SESSION_ID_HEADER)
                .map(|k| k.key.as_str()),
            Some("sess_123")
        );
    }

    #[test]
    fn default_user_agent_uses_crate_version() {
        let _g = env_guard();
        clear_env();

        let client = Client::builder()
            .api_token("hd_x")
            .workspace_id("ws_x")
            .build()
            .expect("build ok");

        let ua = client.configuration().user_agent.clone().unwrap();
        assert_eq!(ua, format!("hotdata-rust/{}", env!("CARGO_PKG_VERSION")));
    }
}
