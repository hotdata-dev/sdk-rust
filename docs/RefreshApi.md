# \RefreshApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**refresh_handler**](RefreshApi.md#refresh_handler) | **POST** /v1/refresh | Refresh connection data



## refresh_handler

> models::RefreshResponse refresh_handler(refresh_request)
Refresh connection data

Refresh schema metadata or table data. The behavior depends on the request fields:  - **Schema refresh (all)**: omit all fields — re-discovers tables for every connection. - **Schema refresh (single)**: set `connection_id` — re-discovers tables for one connection. - **Data refresh (single table)**: set `connection_id`, `schema_name`, `table_name`, and `data: true`. - **Data refresh (connection)**: set `connection_id` and `data: true` — refreshes all cached tables. Set `include_uncached: true` to also sync tables that haven't been cached yet.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**refresh_request** | [**RefreshRequest**](RefreshRequest.md) |  | [required] |

### Return type

[**models::RefreshResponse**](RefreshResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

