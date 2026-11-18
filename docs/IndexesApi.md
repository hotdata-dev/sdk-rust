# \IndexesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_index**](IndexesApi.md#create_index) | **POST** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes | Create an index on a table
[**delete_index**](IndexesApi.md#delete_index) | **DELETE** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes/{index_name} | Delete an index
[**list_indexes**](IndexesApi.md#list_indexes) | **GET** /v1/connections/{connection_id}/tables/{schema}/{table}/indexes | List indexes on a table
[**list_indexes_collection**](IndexesApi.md#list_indexes_collection) | **GET** /v1/indexes | List indexes across tables in a database



## create_index

> models::IndexInfoResponse create_index(connection_id, schema, table, create_index_request)
Create an index on a table

Create a sorted, BM25 full-text or vector index on a table. The table must hold loaded data and must not declare `sorted_by`.  A table can have only one vector index. A vector index that generates its own embeddings (with `embedding_provider_id`) must be the only index on its table. Requests that break these rules return 400, even with `async` set.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | The database's `default_connection_id` | [required] |
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


## delete_index

> delete_index(connection_id, schema, table, index_name)
Delete an index

Delete an index from a table. Returns 409 while a load into the table is running; retry when it finishes. Deleting a vector index that generated its own embeddings keeps the embedding column in the table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | The database's `default_connection_id` | [required] |
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


## list_indexes

> models::ListIndexesResponse list_indexes(connection_id, schema, table)
List indexes on a table

List the indexes on a table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | The database's `default_connection_id` | [required] |
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

List the indexes in the database named by the `X-Database-Id` header, one page at a time. Filter by connection, schema, table or index type.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_database_id** | **String** | Database ID. Required. | [required] |
**connection_id** | Option<**String**> | Filter by connection ID |  |
**schema** | Option<**String**> | Filter by schema name |  |
**table** | Option<**String**> | Filter by table name |  |
**index_type** | Option<**String**> | Filter by index type: `sorted`, `bm25` or `vector` |  |
**limit** | Option<**i32**> | Indexes per page. Default 100, maximum 1000. |  |
**cursor** | Option<**String**> | The `next_cursor` from the previous page |  |

### Return type

[**models::ListIndexesPageResponse**](ListIndexesPageResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

