# \IndexesApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_index**](IndexesApi.md#create_index) | **POST** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes | Create an index on a table
[**delete_index**](IndexesApi.md#delete_index) | **DELETE** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes/{index_name} | Delete an index
[**list_indexes**](IndexesApi.md#list_indexes) | **GET** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes | List indexes on a table



## create_index

> models::IndexInfoResponse create_index(connection_id, schema, table, create_index_request)
Create an index on a table

Create a sorted or BM25 full-text index on a cached table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection identifier | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |
**create_index_request** | [**CreateIndexRequest**](CreateIndexRequest.md) |  | [required] |

### Return type

[**models::IndexInfoResponse**](IndexInfoResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_index

> delete_index(connection_id, schema, table, index_name)
Delete an index

Delete a specific index from a cached table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection identifier | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |
**index_name** | **String** | Index name | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_indexes

> models::ListIndexesResponse list_indexes(connection_id, schema, table)
List indexes on a table

List all indexes created on a cached table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection identifier | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |

### Return type

[**models::ListIndexesResponse**](ListIndexesResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

