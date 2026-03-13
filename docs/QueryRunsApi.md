# \QueryRunsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_query_runs**](QueryRunsApi.md#list_query_runs) | **GET** /v1/query-runs | List query runs



## list_query_runs

> models::ListQueryRunsResponse list_query_runs(limit, cursor)
List query runs

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of results |  |
**cursor** | Option<**String**> | Pagination cursor |  |

### Return type

[**models::ListQueryRunsResponse**](ListQueryRunsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

