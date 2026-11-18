# \QueryRunsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_query_run**](QueryRunsApi.md#get_query_run) | **GET** /v1/query-runs/{id} | Get query run
[**list_query_runs**](QueryRunsApi.md#list_query_runs) | **GET** /v1/query-runs | List query runs



## get_query_run

> models::QueryRunInfo get_query_run(id, x_database_id)
Get query run

Get a query run's status and details. Send the `X-Database-Id` header of the database the query ran in.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Query run ID | [required] |
**x_database_id** | **String** | ID of the database the query ran in. Required. | [required] |

### Return type

[**models::QueryRunInfo**](QueryRunInfo.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_query_runs

> models::ListQueryRunsResponse list_query_runs(x_database_id, limit, cursor, status, saved_query_id)
List query runs

List the query runs in the database named by the `X-Database-Id` header, newest first, one page at a time. If `has_more` is true, pass `next_cursor` as `cursor` to get the next page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_database_id** | **String** | Database ID. Required. | [required] |
**limit** | Option<**i32**> | Runs per page. Default 100, maximum 1000. |  |
**cursor** | Option<**String**> | The `next_cursor` from the previous page |  |
**status** | Option<**String**> | Filter by status, comma-separated, for example `status=running,failed`. Values: `running`, `succeeded`, `failed`, `interrupted`. |  |
**saved_query_id** | Option<**String**> | Filter by saved query ID |  |

### Return type

[**models::ListQueryRunsResponse**](ListQueryRunsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

