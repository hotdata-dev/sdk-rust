# \QueryApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**query**](QueryApi.md#query) | **POST** /v1/query | Execute SQL query



## query

> models::QueryResponse query(query_request, x_database_id)
Execute SQL query

Execute a SQL query scoped to a database. A database is the only window into catalogs: the query sees only that database's auto `default` catalog plus any catalogs explicitly attached to it. Select the database with EITHER the `X-Database-Id` header OR the `database_id` body field (exactly one must be given; if both are sent and disagree, that's a 400). Use standard Postgres-compatible SQL; reference the default catalog as `default.<schema>.<table>` (or just `<schema>.<table>` / `<table>`) and attached catalogs by their alias. Results are returned inline and a `result_id` is provided for later retrieval via the Results API.  Set `async: true` to execute asynchronously — returns a query run ID for polling. Optionally set `async_after_ms` to attempt synchronous execution first, falling back to async if the query exceeds the timeout.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query_request** | [**QueryRequest**](QueryRequest.md) |  | [required] |
**x_database_id** | Option<**String**> | Database id to scope the query to. Required unless the `database_id` body field is set; if both are present they must match. Only that database's catalogs are visible during planning. A malformed value is a 400; an unknown database id is a 404. |  |

### Return type

[**models::QueryResponse**](QueryResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [SessionId](../README.md#SessionId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

