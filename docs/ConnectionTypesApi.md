# \ConnectionTypesApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_connection_type**](ConnectionTypesApi.md#get_connection_type) | **GET** /v1/connection-types/{name} | Get connection type details
[**list_connection_types**](ConnectionTypesApi.md#list_connection_types) | **GET** /v1/connection-types | List connection types



## get_connection_type

> models::ConnectionTypeDetail get_connection_type(name)
Get connection type details

Get configuration schema and authentication requirements for a specific connection type.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Connection type name (e.g. postgres, mysql, snowflake) | [required] |

### Return type

[**models::ConnectionTypeDetail**](ConnectionTypeDetail.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_connection_types

> models::ListConnectionTypesResponse list_connection_types()
List connection types

List all available connection types, including native sources and FlightDLT services.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListConnectionTypesResponse**](ListConnectionTypesResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

