# \RefreshApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**refresh**](RefreshApi.md#refresh) | **POST** /v1/refresh | Refresh connection data



## refresh

> models::RefreshResponse refresh(refresh_request)
Refresh connection data

Refresh schema metadata or table data. The behavior depends on the request fields:  - **Schema refresh (all)**: omit all fields — re-discovers tables for every connection. - **Schema refresh (single)**: set `connection_id` — re-discovers tables for one connection. - **Data refresh (single table)**: set `connection_id`, `schema_name`, `table_name`, and `data: true`. - **Data refresh (connection)**: set `connection_id` and `data: true` — refreshes all cached tables. Set `include_uncached: true` to also sync tables that haven't been cached yet.  Set `async: true` on data refresh operations to run in the background and return a job ID for polling.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**refresh_request** | [**RefreshRequest**](RefreshRequest.md) |  | [required] |

### Return type

[**models::RefreshResponse**](RefreshResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

