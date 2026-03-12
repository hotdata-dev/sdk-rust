# \ResultsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_result_handler**](ResultsApi.md#get_result_handler) | **GET** /v1/results/{id} | Get result
[**list_results_handler**](ResultsApi.md#list_results_handler) | **GET** /v1/results | List results



## get_result_handler

> models::GetResultResponse get_result_handler(id)
Get result

Retrieve a persisted query result by ID. If the result is still being processed, only the status is returned. Once ready, the full column and row data is included in the response.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Result ID | [required] |

### Return type

[**models::GetResultResponse**](GetResultResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_results_handler

> models::ListResultsResponse list_results_handler(limit, offset)
List results

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of results (default: 100, max: 1000) |  |
**offset** | Option<**i32**> | Pagination offset (default: 0) |  |

### Return type

[**models::ListResultsResponse**](ListResultsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

