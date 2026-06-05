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
use crate::auth::{TokenManager, TokenManagerOptions};
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
    client_id: Option<String>,
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

    /// Override the `client_id` sent with every token-exchange (JWT mint)
    /// request. Defaults to the SDK's identifier (`hotdata-rust-sdk`). Set this
    /// so token traffic is attributed to the host application (e.g.
    /// `hotdata-cli`) rather than the SDK.
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
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
        // host, honoring any test override). A caller-supplied client_id
        // attributes the token traffic to the host app; otherwise the SDK
        // default applies.
        let token_manager = TokenManager::with_options(
            api_token,
            http_client,
            TokenManagerOptions {
                base_path,
                client_id: self
                    .client_id
                    .clone()
                    .unwrap_or_else(|| TokenManagerOptions::default().client_id),
                ..TokenManagerOptions::default()
            },
        );
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

    /// Submit a query to `POST /v1/query`, surfacing BOTH response shapes the
    /// server can return.
    ///
    /// The generated [`apis::query_api::query`](crate::apis::query_api::query)
    /// op only models the `200 QueryResponse` branch — the generator collapsed
    /// the `202 AsyncQueryResponse` branch, so callers can't recover the
    /// `query_run_id` when a query goes async. This method closes that gap:
    ///
    /// - HTTP `200` decodes the inline [`models::QueryResponse`] into
    ///   [`QueryOutcome::Inline`].
    /// - HTTP `202` decodes the [`models::AsyncQueryResponse`] (carrying
    ///   `query_run_id` / `status` / `status_url`) into
    ///   [`QueryOutcome::Submitted`]; poll it via
    ///   [`Client::list_query_runs`] / the query-runs API.
    /// - Any other status maps to the SDK's existing
    ///   [`Error::ResponseError`](crate::apis::Error) /
    ///   [`ResponseContent`](crate::apis::ResponseContent) shape with a
    ///   [`apis::query_api::QueryError`] entity, identical to the generated op.
    ///
    /// `database_id` selects the database via the `X-Database-Id` header (the
    /// spec lets database scope come from that header OR the `database_id` body
    /// field; pass `None` to use the body field or rely on the default). The
    /// request is built wire-identically to the generated op (same workspace /
    /// session scope headers, same bearer auth, same `base_path`/`/v1` join,
    /// same JSON body) plus the `202` handling.
    pub async fn submit_query(
        &self,
        request: models::QueryRequest,
        database_id: Option<&str>,
    ) -> Result<QueryOutcome, Error<apis::query_api::QueryError>> {
        use crate::apis::ResponseContent;

        let configuration = &self.configuration;

        let uri_str = format!("{}/v1/query", configuration.base_path);
        let mut req_builder = configuration
            .client
            .request(reqwest::Method::POST, &uri_str);

        if let Some(ref user_agent) = configuration.user_agent {
            req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
        }
        if let Some(param_value) = database_id {
            req_builder = req_builder.header("X-Database-Id", param_value.to_string());
        }
        if let Some(apikey) = configuration.api_keys.get("X-Workspace-Id") {
            let key = apikey.key.clone();
            let value = match apikey.prefix {
                Some(ref prefix) => format!("{} {}", prefix, key),
                None => key,
            };
            req_builder = req_builder.header("X-Workspace-Id", value);
        };
        if let Some(apikey) = configuration.api_keys.get("X-Session-Id") {
            let key = apikey.key.clone();
            let value = match apikey.prefix {
                Some(ref prefix) => format!("{} {}", prefix, key),
                None => key,
            };
            req_builder = req_builder.header("X-Session-Id", value);
        };
        if let Some(token) = configuration.resolve_bearer_token().await {
            req_builder = req_builder.bearer_auth(token);
        };
        req_builder = req_builder.json(&request);

        let req = req_builder.build()?;
        crate::http_log::log_request(&req);
        let resp = configuration.client.execute(req).await?;

        let status = resp.status();
        crate::http_log::log_response_status(status);

        if status == reqwest::StatusCode::ACCEPTED {
            // 202 Accepted: the query was submitted asynchronously.
            let content = resp.text().await?;
            crate::http_log::log_response_body(&content);
            let submitted: models::AsyncQueryResponse = serde_json::from_str(&content)?;
            Ok(QueryOutcome::Submitted(submitted))
        } else if !status.is_client_error() && !status.is_server_error() {
            // 2xx (typically 200): inline results.
            let content = resp.text().await?;
            crate::http_log::log_response_body(&content);
            let inline: models::QueryResponse = serde_json::from_str(&content)?;
            Ok(QueryOutcome::Inline(inline))
        } else {
            let content = resp.text().await?;
            crate::http_log::log_response_body(&content);
            let entity: Option<apis::query_api::QueryError> = serde_json::from_str(&content).ok();
            Err(Error::ResponseError(ResponseContent {
                status,
                content,
                entity,
            }))
        }
    }

    /// Stream an arbitrary byte source to `POST /v1/files`, the raw-body upload
    /// endpoint.
    ///
    /// The generated [`apis::uploads_api::upload_file`] (and the ergonomic
    /// `uploads().upload`) take a [`std::path::PathBuf`] only — they open a file
    /// and wrap it in a stream internally. That can't express an arbitrary byte
    /// source (a network download, an in-memory buffer, a transformed stream),
    /// which a host app needs to upload from a `--url` fetch or to wrap its
    /// source in a progress meter. This method closes that gap: it takes any
    /// `Stream` of `Result<Bytes, _>` and streams it to the same endpoint with
    /// the same wire format (raw `application/octet-stream` body, the upload ID
    /// returned in [`models::UploadResponse`]).
    ///
    /// The request mirrors the generated op: same `base_path` + `/v1/files`
    /// join, same `X-Workspace-Id` scope header (from `configuration.api_keys`),
    /// same bearer auth (via [`Configuration::resolve_bearer_token`]), same
    /// user-agent. It additionally sends the `X-Session-Id` scope header and an
    /// explicit `Content-Type` (see `content_type` below) — neither of which the
    /// generated `upload_file` sets — so a session-scoped, typed upload is
    /// expressible. It reuses the configured `configuration.client`, so a
    /// caller-supplied client (e.g. one
    /// built with no request timeout via
    /// [`ClientBuilder::reqwest_client`](crate::ClientBuilder)) applies to the
    /// transfer.
    ///
    /// Progress reporting and timeout policy stay with the caller: wrap the
    /// source stream to drive a progress bar, and supply a no-timeout client if
    /// the upload may outlive the default request timeout. This method owns
    /// neither.
    ///
    /// `content_type` sets the `Content-Type` header (e.g. `text/csv`,
    /// `application/parquet`); pass `None` to default to
    /// `application/octet-stream`. The endpoint accepts the raw bytes as-is.
    pub async fn upload_stream<S, B, E>(
        &self,
        body: S,
        content_type: Option<&str>,
    ) -> Result<models::UploadResponse, Error<apis::uploads_api::UploadFileError>>
    where
        S: futures_core::Stream<Item = Result<B, E>> + Send + 'static,
        bytes::Bytes: From<B>,
        B: 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    {
        use crate::apis::ResponseContent;
        use serde::de::Error as _;

        let configuration = &self.configuration;

        let uri_str = format!("{}/v1/files", configuration.base_path);
        let mut req_builder = configuration
            .client
            .request(reqwest::Method::POST, &uri_str);

        if let Some(ref user_agent) = configuration.user_agent {
            req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
        }
        req_builder = req_builder.header(
            reqwest::header::CONTENT_TYPE,
            content_type.unwrap_or("application/octet-stream"),
        );
        if let Some(apikey) = configuration.api_keys.get("X-Workspace-Id") {
            let key = apikey.key.clone();
            let value = match apikey.prefix {
                Some(ref prefix) => format!("{} {}", prefix, key),
                None => key,
            };
            req_builder = req_builder.header("X-Workspace-Id", value);
        };
        if let Some(apikey) = configuration.api_keys.get("X-Session-Id") {
            let key = apikey.key.clone();
            let value = match apikey.prefix {
                Some(ref prefix) => format!("{} {}", prefix, key),
                None => key,
            };
            req_builder = req_builder.header("X-Session-Id", value);
        };
        if let Some(token) = configuration.resolve_bearer_token().await {
            req_builder = req_builder.bearer_auth(token);
        };
        req_builder = req_builder.body(reqwest::Body::wrap_stream(body));

        let req = req_builder.build()?;
        crate::http_log::log_request(&req);
        let resp = configuration.client.execute(req).await?;

        let status = resp.status();
        crate::http_log::log_response_status(status);
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let is_json =
            content_type.starts_with("application") && content_type.contains("json");

        if !status.is_client_error() && !status.is_server_error() {
            let content = resp.text().await?;
            crate::http_log::log_response_body(&content);
            if is_json {
                serde_json::from_str(&content).map_err(Error::from)
            } else {
                Err(Error::from(serde_json::Error::custom(format!(
                    "Received `{content_type}` content type response that cannot be converted to `models::UploadResponse`"
                ))))
            }
        } else {
            let content = resp.text().await?;
            crate::http_log::log_response_body(&content);
            let entity: Option<apis::uploads_api::UploadFileError> =
                serde_json::from_str(&content).ok();
            Err(Error::ResponseError(ResponseContent {
                status,
                content,
                entity,
            }))
        }
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
            match crate::status::ResultStatus::parse(&result.status) {
                crate::status::ResultStatus::Ready => return Ok(result),
                crate::status::ResultStatus::Failed => {
                    return Err(AwaitResultError::Failed {
                        result_id: result_id.to_owned(),
                        error_message: result.error_message.flatten(),
                    })
                }
                // Pending / Processing / unknown -> keep polling.
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
    /// Runs the query, then polls [`get_result_arrow`](Client::get_result_arrow)
    /// directly — retrying while the result is still pending — until it is
    /// `ready` and decodes the Arrow IPC stream. Polling the Arrow endpoint
    /// (rather than [`await_result`](Client::await_result)) avoids downloading
    /// the full result as JSON on every poll. Returns
    /// [`QueryToArrowError::NoResultId`] when the query could not be persisted
    /// (no `result_id`, e.g. catalog registration failed), or
    /// [`QueryToArrowError::Timeout`] if it does not become ready within
    /// `poll.timeout`. Requires the `arrow` cargo feature.
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
        // Poll the Arrow endpoint directly, retrying while the result is still
        // pending (HTTP 202 -> `ArrowError::NotReady`). This avoids the full
        // JSON download that `await_result` -> `get_result` performs on every
        // poll: `get_result` returns the entire `rows` payload, so polling it
        // would fetch the whole result set as JSON and then download it a
        // second time as Arrow.
        let deadline = std::time::Instant::now() + poll.timeout;
        loop {
            match self.get_result_arrow(&result_id, offset, limit).await {
                Ok(result) => return Ok(result),
                Err(crate::arrow::ArrowError::NotReady {
                    status,
                    retry_after,
                    ..
                }) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(QueryToArrowError::Timeout {
                            result_id,
                            last_status: status,
                            waited: poll.timeout,
                        });
                    }
                    // Honor the server's `Retry-After` when present, otherwise
                    // fall back to the configured poll interval, but never sleep
                    // past the deadline (a large `Retry-After` must not overshoot
                    // `poll.timeout`).
                    let wait = retry_after
                        .map(std::time::Duration::from_secs)
                        .unwrap_or(poll.interval)
                        .min(deadline.saturating_duration_since(std::time::Instant::now()));
                    tokio::time::sleep(wait).await;
                }
                Err(e) => return Err(QueryToArrowError::Arrow(e)),
            }
        }
    }
}

/// The two response shapes [`Client::submit_query`] can return.
///
/// `POST /v1/query` returns EITHER inline results (`200 QueryResponse`) or an
/// async-submission acknowledgement (`202 AsyncQueryResponse`) depending on
/// whether the query ran synchronously. This enum surfaces both so callers can
/// recover the `query_run_id` and poll when a query goes async.
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum QueryOutcome {
    /// The query ran synchronously and rows were returned inline (HTTP 200).
    Inline(models::QueryResponse),
    /// The query was submitted asynchronously (HTTP 202); poll its
    /// `query_run_id` for completion.
    Submitted(models::AsyncQueryResponse),
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
    /// The result did not become `ready` before `poll.timeout` elapsed.
    Timeout {
        /// The result id being awaited.
        result_id: String,
        /// The last status observed before timing out.
        last_status: String,
        /// How long was waited before giving up.
        waited: std::time::Duration,
    },
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
            QueryToArrowError::Timeout {
                result_id,
                last_status,
                waited,
            } => write!(
                f,
                "result {result_id} not ready after {waited:?} (last status: {last_status})"
            ),
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
        // Shared crate-wide lock (see crate::ENV_LOCK) so these env-mutating
        // tests serialize against auth.rs's env tests too — env is global.
        crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn clear_env() {
        for key in [
            ENV_API_KEY,
            ENV_WORKSPACE_ID,
            ENV_API_URL,
            ENV_TEST_API_URL,
            "HOTDATA_SESSION_ID",
            "HOTDATA_DISABLE_JWT_EXCHANGE",
        ] {
            env::remove_var(key);
        }
    }

    /// The builder's `client_id` override must reach the token-exchange wire so
    /// host apps (e.g. the CLI) attribute their token traffic correctly.
    #[tokio::test]
    async fn builder_client_id_attributes_token_traffic() {
        use wiremock::matchers::{body_string_contains, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/auth/jwt"))
            .and(body_string_contains("client_id=hotdata-cli"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "minted-jwt",
                "expires_in": 300,
                "refresh_token": "r1"
            })))
            .mount(&server)
            .await;

        let client = Client::builder()
            .api_token("hd_opaque")
            .workspace_id("ws_x")
            .client_id("hotdata-cli")
            .base_url(server.uri())
            .build()
            .expect("build should succeed");

        // Driving the token provider forces a mint; the mock only matches when
        // the body carries client_id=hotdata-cli, so a None here means the
        // override did not reach the wire.
        let bearer = client.configuration().resolve_bearer_token().await;
        assert_eq!(bearer.as_deref(), Some("minted-jwt"));

        clear_env();
    }

    /// Helper: build a client pointed at a wiremock server with a static bearer
    /// token (no JWT-exchange round-trip), so query tests assert on the
    /// `/v1/query` request directly.
    fn query_test_client(base_url: &str) -> Client {
        let mut configuration = Configuration {
            base_path: base_url.to_owned(),
            user_agent: Some("hotdata-rust-test".to_owned()),
            bearer_access_token: Some("test-bearer".to_owned()),
            ..Configuration::default()
        };
        configuration.api_keys.insert(
            WORKSPACE_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: "ws_test".to_owned(),
            },
        );
        Client::from_configuration(configuration)
    }

    /// 200 -> inline results decode into `QueryOutcome::Inline`.
    #[tokio::test]
    async fn submit_query_200_inline() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "columns": ["n"],
                "execution_time_ms": 3,
                "nullable": [false],
                "query_run_id": "qr_inline",
                "row_count": 1,
                "rows": [[1]],
            })))
            .mount(&server)
            .await;

        let client = query_test_client(&server.uri());
        let outcome = client
            .submit_query(models::QueryRequest::new("select 1".into()), None)
            .await
            .expect("submit_query should succeed");

        match outcome {
            QueryOutcome::Inline(resp) => {
                assert_eq!(resp.query_run_id, "qr_inline");
                assert_eq!(resp.row_count, 1);
            }
            other => panic!("expected Inline, got {other:?}"),
        }
    }

    /// 202 -> async submission decodes into `QueryOutcome::Submitted` with the
    /// right `query_run_id`.
    #[tokio::test]
    async fn submit_query_202_submitted() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "query_run_id": "qr_async_42",
                "status": "pending",
                "status_url": "https://api.hotdata.dev/v1/query-runs/qr_async_42",
            })))
            .mount(&server)
            .await;

        let client = query_test_client(&server.uri());
        let outcome = client
            .submit_query(
                models::QueryRequest {
                    r#async: Some(true),
                    ..models::QueryRequest::new("select 1".into())
                },
                None,
            )
            .await
            .expect("submit_query should succeed");

        match outcome {
            QueryOutcome::Submitted(resp) => {
                assert_eq!(resp.query_run_id, "qr_async_42");
                assert_eq!(resp.status, "pending");
                assert_eq!(
                    resp.status_url,
                    "https://api.hotdata.dev/v1/query-runs/qr_async_42"
                );
            }
            other => panic!("expected Submitted, got {other:?}"),
        }
    }

    /// The request carries `X-Database-Id` (when passed), `X-Workspace-Id`
    /// (api_keys), and the bearer token — wire-identical to the generated op.
    #[tokio::test]
    async fn submit_query_sends_scope_and_auth_headers() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .and(header("X-Database-Id", "db_123"))
            .and(header("X-Workspace-Id", "ws_test"))
            .and(header("Authorization", "Bearer test-bearer"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "query_run_id": "qr_scoped",
                "status": "pending",
                "status_url": "https://api.hotdata.dev/v1/query-runs/qr_scoped",
            })))
            .mount(&server)
            .await;

        let client = query_test_client(&server.uri());
        // The mock only matches when all three headers are present, so a
        // successful Submitted outcome proves they reached the wire.
        let outcome = client
            .submit_query(
                models::QueryRequest::new("select 1".into()),
                Some("db_123"),
            )
            .await
            .expect("submit_query should succeed with scoped headers");

        assert!(matches!(outcome, QueryOutcome::Submitted(_)));
    }

    /// A streamed body POSTs to `/v1/files`, parses the `201 UploadResponse`,
    /// and the streamed bytes arrive intact.
    #[tokio::test]
    async fn upload_stream_posts_and_parses_response() {
        use wiremock::matchers::{body_bytes, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/files"))
            .and(body_bytes(b"hello world".to_vec()))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "content_type": "text/csv",
                "created_at": "2026-06-04T00:00:00Z",
                "id": "upload_abc",
                "size_bytes": 11,
                "status": "ready",
            })))
            .mount(&server)
            .await;

        let client = query_test_client(&server.uri());
        // Two chunks so the body is genuinely streamed (not a single buffer).
        let chunks: Vec<Result<bytes::Bytes, std::io::Error>> = vec![
            Ok(bytes::Bytes::from_static(b"hello ")),
            Ok(bytes::Bytes::from_static(b"world")),
        ];
        let stream = futures::stream::iter(chunks);

        let resp = client
            .upload_stream(stream, Some("text/csv"))
            .await
            .expect("upload_stream should succeed");

        assert_eq!(resp.id, "upload_abc");
        assert_eq!(resp.size_bytes, 11);
        assert_eq!(resp.status, "ready");
    }

    /// The request carries `X-Workspace-Id` and `Authorization: Bearer` like the
    /// generated `upload_file` op, plus the intentional `Content-Type` header
    /// `upload_file` omits.
    #[tokio::test]
    async fn upload_stream_sends_scope_auth_and_content_type() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/files"))
            .and(header("X-Workspace-Id", "ws_test"))
            .and(header("Authorization", "Bearer test-bearer"))
            .and(header("Content-Type", "application/parquet"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "created_at": "2026-06-04T00:00:00Z",
                "id": "upload_scoped",
                "size_bytes": 3,
                "status": "ready",
            })))
            .mount(&server)
            .await;

        let client = query_test_client(&server.uri());
        // The mock only matches when all three headers are present, so a
        // successful parse proves they reached the wire.
        let stream = futures::stream::once(async {
            Ok::<_, std::io::Error>(bytes::Bytes::from_static(b"abc"))
        });

        let resp = client
            .upload_stream(stream, Some("application/parquet"))
            .await
            .expect("upload_stream should succeed with scoped headers");

        assert_eq!(resp.id, "upload_scoped");
    }

    /// With no `content_type`, the request defaults to
    /// `application/octet-stream`.
    #[tokio::test]
    async fn upload_stream_defaults_content_type() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let _g = env_guard();
        clear_env();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/files"))
            .and(header("Content-Type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "created_at": "2026-06-04T00:00:00Z",
                "id": "upload_default",
                "size_bytes": 1,
                "status": "ready",
            })))
            .mount(&server)
            .await;

        let client = query_test_client(&server.uri());
        let stream = futures::stream::once(async {
            Ok::<_, std::io::Error>(bytes::Bytes::from_static(b"x"))
        });

        let resp = client
            .upload_stream(stream, None)
            .await
            .expect("upload_stream should default content-type");

        assert_eq!(resp.id, "upload_default");
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
