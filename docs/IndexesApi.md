# \IndexesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_dataset_index**](IndexesApi.md#create_dataset_index) | **POST** /v1/datasets/{dataset_id}/indexes | Create an index on a dataset
[**create_index**](IndexesApi.md#create_index) | **POST** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes | Create an index on a table
[**delete_dataset_index**](IndexesApi.md#delete_dataset_index) | **DELETE** /v1/datasets/{dataset_id}/indexes/{index_name} | Delete a dataset index
[**delete_index**](IndexesApi.md#delete_index) | **DELETE** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes/{index_name} | Delete an index
[**list_dataset_indexes**](IndexesApi.md#list_dataset_indexes) | **GET** /v1/datasets/{dataset_id}/indexes | List indexes on a dataset
[**list_indexes**](IndexesApi.md#list_indexes) | **GET** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes | List indexes on a table
[**list_indexes_collection**](IndexesApi.md#list_indexes_collection) | **GET** /v1/indexes | List indexes across tables in a database



## create_dataset_index

> models::IndexInfoResponse create_dataset_index(dataset_id, create_index_request)
Create an index on a dataset

Create a sorted, BM25, or vector index on a dataset.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset identifier | [required] |
**create_index_request** | [**CreateIndexRequest**](CreateIndexRequest.md) |  | [required] |

### Return type

[**models::IndexInfoResponse**](IndexInfoResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [SessionId](../README.md#SessionId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_dataset_index

> delete_dataset_index(dataset_id, index_name)
Delete a dataset index

Delete a specific index from a dataset.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset identifier | [required] |
**index_name** | **String** | Index name | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [SessionId](../README.md#SessionId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
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

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_dataset_indexes

> models::ListIndexesResponse list_dataset_indexes(dataset_id)
List indexes on a dataset

List all indexes created on a dataset.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset identifier | [required] |

### Return type

[**models::ListIndexesResponse**](ListIndexesResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [SessionId](../README.md#SessionId), [BearerAuth](../README.md#BearerAuth)

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

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_indexes_collection

> models::ListIndexesPageResponse list_indexes_collection(x_database_id, connection_id, schema, table, index_type, limit, cursor)
List indexes across tables in a database

List all indexes in the database identified by the required X-Database-Id header, paginated. Optional filters narrow by connection, schema, table, or index type.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_database_id** | **String** | Database to scope to (required) | [required] |
**connection_id** | Option<**String**> | Filter to one connection |  |
**schema** | Option<**String**> | Filter by schema name |  |
**table** | Option<**String**> | Filter by table name |  |
**index_type** | Option<**String**> | Filter by index type |  |
**limit** | Option<**i32**> | Max indexes per page |  |
**cursor** | Option<**String**> | Pagination cursor |  |

### Return type

[**models::ListIndexesPageResponse**](ListIndexesPageResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

