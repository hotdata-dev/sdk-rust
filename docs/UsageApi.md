# \UsageApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_usage**](UsageApi.md#get_usage) | **GET** /v1/usage | Get workspace usage snapshot



## get_usage

> models::WorkspaceUsageResponse get_usage(since)
Get workspace usage snapshot

Get bytes scanned and query count since `since`, plus current storage size. Set `since` to the start of your billing period so the totals match your invoice.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**since** | Option<**String**> | Start of the period, as an ISO 8601 timestamp. Default: the start of the current calendar month, in UTC. |  |

### Return type

[**models::WorkspaceUsageResponse**](WorkspaceUsageResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

