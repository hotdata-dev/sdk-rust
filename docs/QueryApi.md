# \QueryApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**query**](QueryApi.md#query) | **POST** /v1/query | Execute SQL query



## query

> models::QueryResponse query(query_request)
Execute SQL query

Execute a SQL query against all registered connections and datasets. Use standard Postgres-compatible SQL to reference tables from any connection using the format `connection_name.schema.table`. Results are returned inline and a `result_id` is provided for later retrieval via the Results API.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query_request** | [**QueryRequest**](QueryRequest.md) |  | [required] |

### Return type

[**models::QueryResponse**](QueryResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

