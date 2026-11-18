# \ResultsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_result**](ResultsApi.md#get_result) | **GET** /v1/results/{id} | Get result
[**list_results**](ResultsApi.md#list_results) | **GET** /v1/results | List results



## get_result

> models::GetResultResponse get_result(id, x_database_id, offset, limit, format)
Get result

Get a saved query result by ID. Send the `X-Database-Id` header of the database it belongs to. Choose the format with `?format=` (`json`, `arrow`, `csv`, `md` or `parquet`) or the `Accept` header. `?format=` wins; an unknown value is ignored.  | Result status              | Response                                        | |----------------------------|-------------------------------------------------| | `ready`                    | 200 with the rows in the chosen format          | | `pending` or `processing`  | 202 with a `Retry-After` header. Poll again.    | | `failed`                   | 409 with `error_message`                        |  Use `offset` and `limit` to read part of the result. When more rows follow, a `Link` header with `rel=\"next\"` points to the next page. Rows have no guaranteed order, so paging is not a reliable way to read a whole result. Parquet ignores `offset` and `limit` and returns the whole file.  JSON responses have a size limit: a result too large to return as JSON gets 413 `RESULT_TOO_LARGE`. Arrow, CSV, Markdown and Parquet stream with no size limit, so use them for large results. All formats except Parquet carry `X-Total-Row-Count`, the row count of the whole result.  JSON, CSV and Markdown write `NaN` and `±Inf` as `null` or an empty cell. Arrow and Parquet keep the original values.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Result ID | [required] |
**x_database_id** | **String** | ID of the database the result belongs to. Required. | [required] |
**offset** | Option<**i32**> | Rows to skip. Default 0. |  |
**limit** | Option<**i32**> | Maximum rows to return. Default: all rows. |  |
**format** | Option<[**ResultsFormatQuery**](ResultsFormatQuery.md)> | `json`, `arrow`, `csv`, `md` (or `markdown`) or `parquet`. Overrides the `Accept` header. |  |

### Return type

[**models::GetResultResponse**](GetResultResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/vnd.apache.arrow.stream, text/csv, text/markdown, application/vnd.apache.parquet

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_results

> models::ListResultsResponse list_results(x_database_id, limit, offset)
List results

List saved query results in the database named by the `X-Database-Id` header, newest first.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_database_id** | **String** | Database ID. Required. | [required] |
**limit** | Option<**i32**> | Results to return. Default 100, maximum 1000. |  |
**offset** | Option<**i32**> | Number of results to skip. Default 0. |  |

### Return type

[**models::ListResultsResponse**](ListResultsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

