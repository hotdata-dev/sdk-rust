# \QueryRunsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_query_run**](QueryRunsApi.md#get_query_run) | **GET** /v1/query-runs/{id} | Get query run
[**list_query_runs**](QueryRunsApi.md#list_query_runs) | **GET** /v1/query-runs | List query runs



## get_query_run

> models::QueryRunInfo get_query_run(id)
Get query run

Get the status and details of a specific query run by ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Query run ID | [required] |

### Return type

[**models::QueryRunInfo**](QueryRunInfo.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_query_runs

> models::ListQueryRunsResponse list_query_runs(limit, cursor, status, saved_query_id)
List query runs

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of results |  |
**cursor** | Option<**String**> | Pagination cursor |  |
**status** | Option<**String**> | Filter by status (comma-separated, e.g. status=running,failed) |  |
**saved_query_id** | Option<**String**> | Filter by saved query ID |  |

### Return type

[**models::ListQueryRunsResponse**](ListQueryRunsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

