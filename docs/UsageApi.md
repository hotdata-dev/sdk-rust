# \UsageApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_usage**](UsageApi.md#get_usage) | **GET** /v1/usage | Get workspace usage snapshot



## get_usage

> models::WorkspaceUsageResponse get_usage(since)
Get workspace usage snapshot

Return aggregated bytes scanned and current storage size for a billing period. Pass `since` as the subscription's `current_period_start` so the meter value aligns with the Stripe invoice window rather than the calendar month.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**since** | Option<**String**> | Billing period start (ISO-8601). Defaults to the start of the current UTC calendar month when omitted. |  |

### Return type

[**models::WorkspaceUsageResponse**](WorkspaceUsageResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

