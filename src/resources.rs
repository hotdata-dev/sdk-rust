//! Ergonomic, hand-written resource handles for the HotData SDK.
//!
//! This module is regeneration-immune: it is protected by `.openapi-generator-ignore`
//! and is never emitted by the OpenAPI generator. The OpenAPI generator emits the
//! operations as free functions grouped into `apis::*_api` modules; this module
//! restores a Python-like, resource-oriented surface by wrapping each generated
//! `*_api` module in a lightweight handle struct that borrows the
//! [`Configuration`](crate::apis::configuration::Configuration) and forwards each
//! operation to the matching free function.
//!
//! Handles are obtained from [`Client`](crate::Client) accessors, e.g.
//! `client.connections().list().await?`. They hold only a `&Configuration`, so
//! they are cheap to construct and never own state.

use crate::apis;
use crate::apis::configuration::Configuration;
use crate::apis::Error;
use crate::models;

/// Connections resource handle. Wraps [`apis::connections_api`](crate::apis::connections_api).
pub struct ConnectionsApi<'a> {
    config: &'a Configuration,
}

impl<'a> ConnectionsApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create a new connection.
    pub async fn create(
        &self,
        request: models::CreateConnectionRequest,
    ) -> Result<models::CreateConnectionResponse, Error<apis::connections_api::CreateConnectionError>>
    {
        apis::connections_api::create_connection(self.config, request).await
    }

    /// Fetch a connection by id.
    pub async fn get(
        &self,
        connection_id: &str,
    ) -> Result<models::GetConnectionResponse, Error<apis::connections_api::GetConnectionError>>
    {
        apis::connections_api::get_connection(self.config, connection_id).await
    }

    /// List connections.
    pub async fn list(
        &self,
    ) -> Result<models::ListConnectionsResponse, Error<apis::connections_api::ListConnectionsError>>
    {
        apis::connections_api::list_connections(self.config).await
    }

    /// Delete a connection by id.
    pub async fn delete(
        &self,
        connection_id: &str,
    ) -> Result<(), Error<apis::connections_api::DeleteConnectionError>> {
        apis::connections_api::delete_connection(self.config, connection_id).await
    }

    /// Check the health of a connection.
    pub async fn check_health(
        &self,
        connection_id: &str,
    ) -> Result<
        models::ConnectionHealthResponse,
        Error<apis::connections_api::CheckConnectionHealthError>,
    > {
        apis::connections_api::check_connection_health(self.config, connection_id).await
    }

    /// Purge the entire cache for a connection.
    pub async fn purge_cache(
        &self,
        connection_id: &str,
    ) -> Result<(), Error<apis::connections_api::PurgeConnectionCacheError>> {
        apis::connections_api::purge_connection_cache(self.config, connection_id).await
    }

    /// Get a profile for a managed table.
    pub async fn get_table_profile(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
    ) -> Result<models::TableProfileResponse, Error<apis::connections_api::GetTableProfileError>>
    {
        apis::connections_api::get_table_profile(self.config, connection_id, schema, table).await
    }

    /// Load (materialize) a managed table.
    pub async fn load_managed_table(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        request: models::LoadManagedTableRequest,
    ) -> Result<models::LoadManagedTableResponse, Error<apis::connections_api::LoadManagedTableError>>
    {
        apis::connections_api::load_managed_table(
            self.config,
            connection_id,
            schema,
            table,
            request,
        )
        .await
    }

    /// Delete a managed table.
    pub async fn delete_managed_table(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
    ) -> Result<(), Error<apis::connections_api::DeleteManagedTableError>> {
        apis::connections_api::delete_managed_table(self.config, connection_id, schema, table).await
    }

    /// Purge the cache for a single table.
    pub async fn purge_table_cache(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
    ) -> Result<(), Error<apis::connections_api::PurgeTableCacheError>> {
        apis::connections_api::purge_table_cache(self.config, connection_id, schema, table).await
    }
}

/// Connection-types resource handle. Wraps [`apis::connection_types_api`](crate::apis::connection_types_api).
pub struct ConnectionTypesApi<'a> {
    config: &'a Configuration,
}

impl<'a> ConnectionTypesApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Get the configuration schema for a connection type by name.
    pub async fn get(
        &self,
        name: &str,
    ) -> Result<
        models::ConnectionTypeDetail,
        Error<apis::connection_types_api::GetConnectionTypeError>,
    > {
        apis::connection_types_api::get_connection_type(self.config, name).await
    }

    /// List available connection types.
    pub async fn list(
        &self,
    ) -> Result<
        models::ListConnectionTypesResponse,
        Error<apis::connection_types_api::ListConnectionTypesError>,
    > {
        apis::connection_types_api::list_connection_types(self.config).await
    }
}

/// Database-context resource handle. Wraps [`apis::database_context_api`](crate::apis::database_context_api).
pub struct DatabaseContextApi<'a> {
    config: &'a Configuration,
}

impl<'a> DatabaseContextApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Fetch a named context document for a database.
    pub async fn get(
        &self,
        database_id: &str,
        name: &str,
    ) -> Result<
        models::GetDatabaseContextResponse,
        Error<apis::database_context_api::GetDatabaseContextError>,
    > {
        apis::database_context_api::get_database_context(self.config, database_id, name).await
    }

    /// List context documents for a database.
    pub async fn list(
        &self,
        database_id: &str,
    ) -> Result<
        models::ListDatabaseContextsResponse,
        Error<apis::database_context_api::ListDatabaseContextsError>,
    > {
        apis::database_context_api::list_database_contexts(self.config, database_id).await
    }

    /// Store (or replace) a named context document for a database.
    pub async fn upsert(
        &self,
        database_id: &str,
        request: models::UpsertDatabaseContextRequest,
    ) -> Result<
        models::UpsertDatabaseContextResponse,
        Error<apis::database_context_api::UpsertDatabaseContextError>,
    > {
        apis::database_context_api::upsert_database_context(self.config, database_id, request).await
    }

    /// Delete a named context document from a database.
    pub async fn delete(
        &self,
        database_id: &str,
        name: &str,
    ) -> Result<(), Error<apis::database_context_api::DeleteDatabaseContextError>> {
        apis::database_context_api::delete_database_context(self.config, database_id, name).await
    }
}

/// Databases resource handle. Wraps [`apis::databases_api`](crate::apis::databases_api).
pub struct DatabasesApi<'a> {
    config: &'a Configuration,
}

impl<'a> DatabasesApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create a new database.
    pub async fn create(
        &self,
        request: models::CreateDatabaseRequest,
    ) -> Result<models::CreateDatabaseResponse, Error<apis::databases_api::CreateDatabaseError>>
    {
        apis::databases_api::create_database(self.config, request).await
    }

    /// Fork a database into a new, independent database.
    pub async fn fork(
        &self,
        database_id: &str,
        request: models::ForkDatabaseRequest,
    ) -> Result<models::CreateDatabaseResponse, Error<apis::databases_api::ForkDatabaseError>> {
        apis::databases_api::fork_database(self.config, database_id, request).await
    }

    /// Fetch a database by id.
    pub async fn get(
        &self,
        database_id: &str,
    ) -> Result<models::DatabaseDetailResponse, Error<apis::databases_api::GetDatabaseError>> {
        apis::databases_api::get_database(self.config, database_id).await
    }

    /// List databases.
    pub async fn list(
        &self,
    ) -> Result<models::ListDatabasesResponse, Error<apis::databases_api::ListDatabasesError>> {
        apis::databases_api::list_databases(self.config).await
    }

    /// Delete a database by id.
    pub async fn delete(
        &self,
        database_id: &str,
    ) -> Result<(), Error<apis::databases_api::DeleteDatabaseError>> {
        apis::databases_api::delete_database(self.config, database_id).await
    }

    /// Attach a catalog (connection) to a database.
    pub async fn attach_catalog(
        &self,
        database_id: &str,
        request: models::AttachDatabaseCatalogRequest,
    ) -> Result<(), Error<apis::databases_api::AttachDatabaseCatalogError>> {
        apis::databases_api::attach_database_catalog(self.config, database_id, request).await
    }

    /// Detach a catalog (connection) from a database.
    pub async fn detach_catalog(
        &self,
        database_id: &str,
        connection_id: &str,
    ) -> Result<(), Error<apis::databases_api::DetachDatabaseCatalogError>> {
        apis::databases_api::detach_database_catalog(self.config, database_id, connection_id).await
    }
}

/// Embedding-providers resource handle. Wraps [`apis::embedding_providers_api`](crate::apis::embedding_providers_api).
pub struct EmbeddingProvidersApi<'a> {
    config: &'a Configuration,
}

impl<'a> EmbeddingProvidersApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create a new embedding provider.
    pub async fn create(
        &self,
        request: models::CreateEmbeddingProviderRequest,
    ) -> Result<
        models::CreateEmbeddingProviderResponse,
        Error<apis::embedding_providers_api::CreateEmbeddingProviderError>,
    > {
        apis::embedding_providers_api::create_embedding_provider(self.config, request).await
    }

    /// Fetch an embedding provider by id.
    pub async fn get(
        &self,
        id: &str,
    ) -> Result<
        models::EmbeddingProviderResponse,
        Error<apis::embedding_providers_api::GetEmbeddingProviderError>,
    > {
        apis::embedding_providers_api::get_embedding_provider(self.config, id).await
    }

    /// List embedding providers.
    pub async fn list(
        &self,
    ) -> Result<
        models::ListEmbeddingProvidersResponse,
        Error<apis::embedding_providers_api::ListEmbeddingProvidersError>,
    > {
        apis::embedding_providers_api::list_embedding_providers(self.config).await
    }

    /// Update an embedding provider by id.
    pub async fn update(
        &self,
        id: &str,
        request: models::UpdateEmbeddingProviderRequest,
    ) -> Result<
        models::UpdateEmbeddingProviderResponse,
        Error<apis::embedding_providers_api::UpdateEmbeddingProviderError>,
    > {
        apis::embedding_providers_api::update_embedding_provider(self.config, id, request).await
    }

    /// Delete an embedding provider by id.
    pub async fn delete(
        &self,
        id: &str,
    ) -> Result<(), Error<apis::embedding_providers_api::DeleteEmbeddingProviderError>> {
        apis::embedding_providers_api::delete_embedding_provider(self.config, id).await
    }
}

/// Indexes resource handle. Wraps [`apis::indexes_api`](crate::apis::indexes_api).
///
/// Covers managed-table (connection/schema/table) indexes.
pub struct IndexesApi<'a> {
    config: &'a Configuration,
}

impl<'a> IndexesApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create an index on a managed table.
    pub async fn create_index(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        request: models::CreateIndexRequest,
    ) -> Result<models::IndexInfoResponse, Error<apis::indexes_api::CreateIndexError>> {
        apis::indexes_api::create_index(self.config, connection_id, schema, table, request).await
    }

    /// List indexes on a managed table.
    pub async fn list_indexes(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
    ) -> Result<models::ListIndexesResponse, Error<apis::indexes_api::ListIndexesError>> {
        apis::indexes_api::list_indexes(self.config, connection_id, schema, table).await
    }

    /// Delete an index from a managed table.
    pub async fn delete_index(
        &self,
        connection_id: &str,
        schema: &str,
        table: &str,
        index_name: &str,
    ) -> Result<(), Error<apis::indexes_api::DeleteIndexError>> {
        apis::indexes_api::delete_index(self.config, connection_id, schema, table, index_name).await
    }
}

/// Information-schema resource handle. Wraps [`apis::information_schema_api`](crate::apis::information_schema_api).
pub struct InformationSchemaApi<'a> {
    config: &'a Configuration,
}

impl<'a> InformationSchemaApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Query the information schema, optionally filtered by connection/schema/table.
    #[allow(clippy::too_many_arguments)]
    pub async fn get(
        &self,
        connection_id: Option<&str>,
        schema: Option<&str>,
        table: Option<&str>,
        include_columns: Option<bool>,
        limit: Option<i32>,
        cursor: Option<&str>,
    ) -> Result<
        models::InformationSchemaResponse,
        Error<apis::information_schema_api::InformationSchemaError>,
    > {
        apis::information_schema_api::information_schema(
            self.config,
            connection_id,
            schema,
            table,
            include_columns,
            limit,
            cursor,
        )
        .await
    }
}

/// Jobs resource handle. Wraps [`apis::jobs_api`](crate::apis::jobs_api).
pub struct JobsApi<'a> {
    config: &'a Configuration,
}

impl<'a> JobsApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Fetch a job by id.
    pub async fn get(
        &self,
        id: &str,
    ) -> Result<models::JobStatusResponse, Error<apis::jobs_api::GetJobError>> {
        apis::jobs_api::get_job(self.config, id).await
    }

    /// List jobs, optionally filtered by type and status.
    pub async fn list(
        &self,
        job_type: Option<models::JobType>,
        status: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<models::ListJobsResponse, Error<apis::jobs_api::ListJobsError>> {
        apis::jobs_api::list_jobs(self.config, job_type, status, limit, offset).await
    }
}

/// Query resource handle. Wraps [`apis::query_api`](crate::apis::query_api).
///
/// Also available as the flat [`Client::query`](crate::Client::query) pass-through.
pub struct QueryApi<'a> {
    config: &'a Configuration,
}

impl<'a> QueryApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Execute a SQL query. `x_database_id` scopes the query to a database.
    pub async fn execute(
        &self,
        request: models::QueryRequest,
        x_database_id: Option<&str>,
    ) -> Result<models::QueryResponse, Error<apis::query_api::QueryError>> {
        apis::query_api::query(self.config, request, x_database_id).await
    }
}

/// Query-runs resource handle. Wraps [`apis::query_runs_api`](crate::apis::query_runs_api).
pub struct QueryRunsApi<'a> {
    config: &'a Configuration,
}

impl<'a> QueryRunsApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Fetch a query run by id. `x_database_id` scopes the lookup to a database
    /// (the required `X-Database-Id` header).
    pub async fn get(
        &self,
        id: &str,
        x_database_id: &str,
    ) -> Result<models::QueryRunInfo, Error<apis::query_runs_api::GetQueryRunError>> {
        apis::query_runs_api::get_query_run(self.config, id, x_database_id).await
    }

    /// List recent query runs for the database named by `x_database_id` (the
    /// required `X-Database-Id` header).
    pub async fn list(
        &self,
        x_database_id: &str,
        limit: Option<i32>,
        cursor: Option<&str>,
        status: Option<&str>,
        saved_query_id: Option<&str>,
    ) -> Result<models::ListQueryRunsResponse, Error<apis::query_runs_api::ListQueryRunsError>>
    {
        apis::query_runs_api::list_query_runs(
            self.config,
            x_database_id,
            limit,
            cursor,
            status,
            saved_query_id,
        )
        .await
    }
}

/// Results resource handle. Wraps [`apis::results_api`](crate::apis::results_api).
///
/// For Arrow IPC decoding use [`Client::get_result_arrow`](crate::Client::get_result_arrow)
/// (requires the `arrow` feature).
pub struct ResultsApi<'a> {
    config: &'a Configuration,
}

impl<'a> ResultsApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Fetch a persisted result by id (JSON form). `x_database_id` scopes the
    /// lookup to a database (the required `X-Database-Id` header).
    pub async fn get(
        &self,
        id: &str,
        x_database_id: &str,
        offset: Option<i32>,
        limit: Option<i32>,
        format: Option<models::ResultsFormatQuery>,
    ) -> Result<models::GetResultResponse, Error<apis::results_api::GetResultError>> {
        apis::results_api::get_result(self.config, id, x_database_id, offset, limit, format).await
    }

    /// List persisted results for the database named by `x_database_id` (the
    /// required `X-Database-Id` header).
    pub async fn list(
        &self,
        x_database_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<models::ListResultsResponse, Error<apis::results_api::ListResultsError>> {
        apis::results_api::list_results(self.config, x_database_id, limit, offset).await
    }
}

/// Refresh resource handle. Wraps [`apis::refresh_api`](crate::apis::refresh_api).
pub struct RefreshApi<'a> {
    config: &'a Configuration,
}

impl<'a> RefreshApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Refresh a refreshable resource.
    pub async fn refresh(
        &self,
        request: models::RefreshRequest,
    ) -> Result<models::RefreshResponse, Error<apis::refresh_api::RefreshError>> {
        apis::refresh_api::refresh(self.config, request).await
    }
}

/// Saved-queries resource handle. Wraps [`apis::saved_queries_api`](crate::apis::saved_queries_api).
pub struct SavedQueriesApi<'a> {
    config: &'a Configuration,
}

impl<'a> SavedQueriesApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create a new saved query.
    pub async fn create(
        &self,
        request: models::CreateSavedQueryRequest,
    ) -> Result<models::SavedQueryDetail, Error<apis::saved_queries_api::CreateSavedQueryError>>
    {
        apis::saved_queries_api::create_saved_query(self.config, request).await
    }

    /// Fetch a saved query by id.
    pub async fn get(
        &self,
        id: &str,
    ) -> Result<models::SavedQueryDetail, Error<apis::saved_queries_api::GetSavedQueryError>> {
        apis::saved_queries_api::get_saved_query(self.config, id).await
    }

    /// List saved queries.
    pub async fn list(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<
        models::ListSavedQueriesResponse,
        Error<apis::saved_queries_api::ListSavedQueriesError>,
    > {
        apis::saved_queries_api::list_saved_queries(self.config, limit, offset).await
    }

    /// Update a saved query by id.
    pub async fn update(
        &self,
        id: &str,
        request: models::UpdateSavedQueryRequest,
    ) -> Result<models::SavedQueryDetail, Error<apis::saved_queries_api::UpdateSavedQueryError>>
    {
        apis::saved_queries_api::update_saved_query(self.config, id, request).await
    }

    /// Delete a saved query by id.
    pub async fn delete(
        &self,
        id: &str,
    ) -> Result<(), Error<apis::saved_queries_api::DeleteSavedQueryError>> {
        apis::saved_queries_api::delete_saved_query(self.config, id).await
    }

    /// Execute a saved query. `x_database_id` scopes the execution to a database.
    pub async fn execute(
        &self,
        id: &str,
        x_database_id: &str,
        request: Option<models::ExecuteSavedQueryRequest>,
    ) -> Result<models::QueryResponse, Error<apis::saved_queries_api::ExecuteSavedQueryError>> {
        apis::saved_queries_api::execute_saved_query(self.config, id, x_database_id, request).await
    }

    /// List versions of a saved query.
    pub async fn list_versions(
        &self,
        id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<
        models::ListSavedQueryVersionsResponse,
        Error<apis::saved_queries_api::ListSavedQueryVersionsError>,
    > {
        apis::saved_queries_api::list_saved_query_versions(self.config, id, limit, offset).await
    }
}

/// Secrets resource handle. Wraps [`apis::secrets_api`](crate::apis::secrets_api).
pub struct SecretsApi<'a> {
    config: &'a Configuration,
}

impl<'a> SecretsApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create a new secret.
    pub async fn create(
        &self,
        request: models::CreateSecretRequest,
    ) -> Result<models::CreateSecretResponse, Error<apis::secrets_api::CreateSecretError>> {
        apis::secrets_api::create_secret(self.config, request).await
    }

    /// Fetch a secret by name.
    pub async fn get(
        &self,
        name: &str,
    ) -> Result<models::GetSecretResponse, Error<apis::secrets_api::GetSecretError>> {
        apis::secrets_api::get_secret(self.config, name).await
    }

    /// List secrets.
    pub async fn list(
        &self,
    ) -> Result<models::ListSecretsResponse, Error<apis::secrets_api::ListSecretsError>> {
        apis::secrets_api::list_secrets(self.config).await
    }

    /// Update a secret by name.
    pub async fn update(
        &self,
        name: &str,
        request: models::UpdateSecretRequest,
    ) -> Result<models::UpdateSecretResponse, Error<apis::secrets_api::UpdateSecretError>> {
        apis::secrets_api::update_secret(self.config, name, request).await
    }

    /// Delete a secret by name.
    pub async fn delete(
        &self,
        name: &str,
    ) -> Result<(), Error<apis::secrets_api::DeleteSecretError>> {
        apis::secrets_api::delete_secret(self.config, name).await
    }
}

/// Uploads resource handle. Wraps [`apis::uploads_api`](crate::apis::uploads_api).
pub struct UploadsApi<'a> {
    config: &'a Configuration,
}

impl<'a> UploadsApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Upload a local file directly to object storage (presigned) and finalize
    /// it. The primary upload path; returns the finalized upload (read
    /// `upload_id` from it). See [`Client::upload_file`](crate::Client::upload_file)
    /// for the full contract and [`UploadOptions`](crate::uploads::UploadOptions).
    pub async fn upload_file(
        &self,
        path: impl AsRef<std::path::Path>,
        opts: crate::uploads::UploadOptions,
    ) -> Result<models::FinalizeUploadResponse, crate::uploads::UploadError> {
        crate::uploads::upload_file(self.config, path.as_ref(), opts).await
    }

    /// Stream a file to the legacy raw-body `POST /v1/files` proxy. Prefer
    /// [`upload_file`](Self::upload_file), the presigned direct-to-storage path.
    pub async fn upload(
        &self,
        body: std::path::PathBuf,
    ) -> Result<models::UploadResponse, Error<apis::uploads_api::UploadFileError>> {
        apis::uploads_api::upload_file(self.config, body).await
    }

    /// List uploads, optionally filtered by status.
    pub async fn list(
        &self,
        status: Option<&str>,
    ) -> Result<models::ListUploadsResponse, Error<apis::uploads_api::ListUploadsError>> {
        apis::uploads_api::list_uploads(self.config, status).await
    }
}

/// Workspaces resource handle. Wraps [`apis::workspaces_api`](crate::apis::workspaces_api).
///
/// Also available as the flat [`Client::list_workspaces`](crate::Client::list_workspaces)
/// pass-through.
pub struct WorkspacesApi<'a> {
    config: &'a Configuration,
}

impl<'a> WorkspacesApi<'a> {
    pub(crate) fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    /// Create a new workspace.
    pub async fn create(
        &self,
        request: models::CreateWorkspaceRequest,
    ) -> Result<models::CreateWorkspaceResponse, Error<apis::workspaces_api::CreateWorkspaceError>>
    {
        apis::workspaces_api::create_workspace(self.config, request).await
    }

    /// List workspaces visible to the authenticated principal.
    pub async fn list(
        &self,
        organization_public_id: Option<&str>,
    ) -> Result<models::ListWorkspacesResponse, Error<apis::workspaces_api::ListWorkspacesError>>
    {
        apis::workspaces_api::list_workspaces(self.config, organization_public_id).await
    }

    /// Delete a workspace by public id.
    pub async fn delete(
        &self,
        public_id: &str,
    ) -> Result<(), Error<apis::workspaces_api::DeleteWorkspaceError>> {
        apis::workspaces_api::delete_workspace(self.config, public_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A handle borrows the exact `Configuration` it is constructed from. We
    /// assert by comparing pointer identity, which proves no copy was made.
    #[test]
    fn handle_borrows_provided_config() {
        let config = Configuration::default();
        let handle = ConnectionsApi::new(&config);
        assert!(std::ptr::eq(handle.config, &config));
    }

    /// Several handle types can borrow the same `Configuration` simultaneously
    /// (shared borrows), confirming the handles are zero-cost views.
    #[test]
    fn multiple_handles_share_one_config() {
        let config = Configuration::default();
        let databases = DatabasesApi::new(&config);
        let connections = ConnectionsApi::new(&config);
        let queries = QueryApi::new(&config);

        assert!(std::ptr::eq(databases.config, &config));
        assert!(std::ptr::eq(connections.config, &config));
        assert!(std::ptr::eq(queries.config, &config));
    }

    /// Construct every handle type from a default config to guarantee each
    /// `new` is callable and the structs line up with their modules.
    #[test]
    fn all_handles_constructible() {
        let config = Configuration::default();
        let _ = ConnectionsApi::new(&config);
        let _ = ConnectionTypesApi::new(&config);
        let _ = DatabaseContextApi::new(&config);
        let _ = DatabasesApi::new(&config);
        let _ = EmbeddingProvidersApi::new(&config);
        let _ = IndexesApi::new(&config);
        let _ = InformationSchemaApi::new(&config);
        let _ = JobsApi::new(&config);
        let _ = QueryApi::new(&config);
        let _ = QueryRunsApi::new(&config);
        let _ = ResultsApi::new(&config);
        let _ = RefreshApi::new(&config);
        let _ = SavedQueriesApi::new(&config);
        let _ = SecretsApi::new(&config);
        let _ = UploadsApi::new(&config);
        let _ = WorkspacesApi::new(&config);
    }
}
