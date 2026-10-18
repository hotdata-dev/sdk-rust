# \QueryApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**query**](QueryApi.md#query) | **POST** /v1/query | Execute SQL query



## query

> models::QueryResponse query(query_request, x_database_id)
Execute SQL query

Execute a SQL query scoped to a database. A database is the only window into catalogs: the query sees only that database's auto `default` catalog plus any catalogs explicitly attached to it. Select the database with EITHER the `X-Database-Id` header OR the `database_id` body field (exactly one must be given; if both are sent and disagree, that's a 400). Use standard Postgres-compatible SQL; reference the default catalog as `default.<schema>.<table>` (or just `<schema>.<table>` / `<table>`) and attached catalogs by their alias. Results are returned inline and a `result_id` is provided for later retrieval via the Results API.  Set `async: true` to execute asynchronously — returns a query run ID for polling. Optionally set `async_after_ms` to attempt synchronous execution first, falling back to async if the query exceeds the timeout.  To receive the rows as an Arrow IPC stream instead of JSON, send `Accept: application/vnd.apache.arrow.stream` (without also listing `application/json`). A query that completes within the request then returns 200 with an Arrow IPC stream body (schema, record batches, end-of-stream), and the values the JSON body would carry are sent as response headers: `X-Query-Run-Id`, `X-Result-Id` (when the result was saved), `X-Total-Row-Count`, `X-Execution-Time-Ms`, and `X-Query-Warning` (when there is a warning). Arrow keeps `±Inf` and `NaN` float values as they are. An Arrow response always holds the whole result: if the result is larger than an inline response can hold, the request returns 202 with the query run ticket and an `X-Result-Id` header instead, and the full result is read from `GET /v1/results/{id}?format=arrow` once it is ready. A query that goes async, and every error, return the same JSON bodies as without the header.  The endpoint also runs write statements: `INSERT INTO … VALUES … / SELECT …` (append only), `CREATE TABLE`, `CREATE TABLE … AS SELECT …` and `DROP TABLE`. A write runs once, inline, and answers with a one-row result holding a single `rows_affected` column. Writes need an API key holding both the `read` and `write` permissions, may only target the database's own `default` catalog (attached catalogs stay read-only), and are never run from a saved query. `UPDATE` and `DELETE` are not supported yet; use the load endpoint's keyed `update` and `delete` modes. Every other mutation (`ALTER`, views, `COPY`, transactions, `EXPLAIN` of a write) is refused.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query_request** | [**QueryRequest**](QueryRequest.md) |  | [required] |
**x_database_id** | Option<**String**> | Database id to scope the query to. Required unless the `database_id` body field is set; if both are present they must match. Only that database's catalogs are visible during planning. A malformed value is a 400; an unknown database id is a 404. |  |

### Return type

[**models::QueryResponse**](QueryResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/vnd.apache.arrow.stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

