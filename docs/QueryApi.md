# \QueryApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**query**](QueryApi.md#query) | **POST** /v1/query | Execute SQL query



## query

> models::QueryResponse query(query_request, x_database_id)
Execute SQL query

Run a SQL query against one database. Choose the database with the `X-Database-Id` header or the `database_id` body field; if you send both, they must match. The query sees the database's `default` catalog and any catalogs attached to it, by their alias. Rows come back inline, with a `result_id` for reading the result again from the Results API.  **Async.** Set `async: true` to get a query run ID back at once and poll for the result. Add `async_after_ms` (1000 to 30000) to wait that long for an inline result first.  **Arrow.** Send `Accept: application/vnd.apache.arrow.stream` (without `application/json`) to get the rows as an Arrow IPC stream. The values the JSON body would carry come as `X-Query-Run-Id`, `X-Result-Id`, `X-Total-Row-Count`, `X-Execution-Time-Ms`, `X-Query-Warning` and `X-Rollup` headers. If the result is too large to send inline, you get a 202 with an `X-Result-Id` header; read the full result from `GET /v1/results/{id}?format=arrow`. Async runs and errors still return JSON.  **Writes.** `INSERT`, `UPDATE`, `DELETE`, `CREATE TABLE`, `CREATE TABLE … AS SELECT` and `DROP TABLE` are supported. A write needs an API key with both the `read` and `write` permissions, can only change the database's `default` catalog, and returns one row with a `rows_affected` column. `INSERT` only appends. `UPDATE` and `DELETE` do not support `RETURNING`, `LIMIT` or `ORDER BY`. `UPDATE` also refuses table aliases, `FROM`, and setting a vector-index column, or a key or `constant_per_key` column. Other changes (`ALTER`, views, `COPY`, transactions) are refused.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query_request** | [**QueryRequest**](QueryRequest.md) |  | [required] |
**x_database_id** | Option<**String**> | ID of the database to query. Required unless you set the `database_id` body field. |  |

### Return type

[**models::QueryResponse**](QueryResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, application/vnd.apache.arrow.stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

