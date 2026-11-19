# \ConnectionsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_managed_schema**](ConnectionsApi.md#add_managed_schema) | **POST** /v1/connections/{connection_id}/schemas | Add managed schema
[**add_managed_table**](ConnectionsApi.md#add_managed_table) | **POST** /v1/connections/{connection_id}/schemas/{schema}/tables | Add managed table
[**create_connection**](ConnectionsApi.md#create_connection) | **POST** /v1/connections | Create connection
[**delete_connection**](ConnectionsApi.md#delete_connection) | **DELETE** /v1/connections/{connection_id} | Delete connection
[**delete_managed_table**](ConnectionsApi.md#delete_managed_table) | **DELETE** /v1/connections/{connection_id}/schemas/{schema}/tables/{table} | Delete managed table
[**get_connection**](ConnectionsApi.md#get_connection) | **GET** /v1/connections/{connection_id} | Get connection
[**get_table_profile**](ConnectionsApi.md#get_table_profile) | **GET** /v1/connections/{connection_id}/tables/{schema}/{table}/profile | Get table profile
[**list_connections**](ConnectionsApi.md#list_connections) | **GET** /v1/connections | List connections
[**load_managed_table**](ConnectionsApi.md#load_managed_table) | **POST** /v1/connections/{connection_id}/schemas/{schema}/tables/{table}/loads | Load managed table from inline data, upload, or query result
[**set_managed_table_constant_per_key**](ConnectionsApi.md#set_managed_table_constant_per_key) | **PUT** /v1/connections/{connection_id}/schemas/{schema}/tables/{table}/constant-per-key | Declare which columns are constant per key



## add_managed_schema

> models::ManagedSchemaResponse add_managed_schema(connection_id, add_managed_schema_request)
Add managed schema

Add a schema, and optionally its tables, to a `managed` connection. Names are stored in lowercase. To add a schema to a database, use `POST /v1/databases/{database_id}/schemas`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**add_managed_schema_request** | [**AddManagedSchemaRequest**](AddManagedSchemaRequest.md) |  | [required] |

### Return type

[**models::ManagedSchemaResponse**](ManagedSchemaResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## add_managed_table

> models::ManagedTableResponse add_managed_table(connection_id, schema, add_managed_table_request)
Add managed table

Add an empty table to a schema in a `managed` connection. Names are stored in lowercase. You do not have to declare tables first: a load creates the table if it is missing.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**add_managed_table_request** | [**AddManagedTableRequest**](AddManagedTableRequest.md) |  | [required] |

### Return type

[**models::ManagedTableResponse**](ManagedTableResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_connection

> models::CreateConnectionResponse create_connection(create_connection_request)
Create connection

Create a catalog that holds data you load into it. `source_type` must be `managed`; other source types are refused with a 400. A managed connection takes no credentials. You can declare its schemas and tables in `config` now, or add them later.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_connection_request** | [**CreateConnectionRequest**](CreateConnectionRequest.md) |  | [required] |

### Return type

[**models::CreateConnectionResponse**](CreateConnectionResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_connection

> delete_connection(connection_id)
Delete connection

Delete a connection and all of its tables and data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_managed_table

> delete_managed_table(connection_id, schema, table)
Delete managed table

Delete a table and its data from a `managed` connection.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_connection

> models::GetConnectionResponse get_connection(connection_id)
Get connection

Get a connection, with its table count and how many of its tables hold loaded data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |

### Return type

[**models::GetConnectionResponse**](GetConnectionResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_table_profile

> models::TableProfileResponse get_table_profile(connection_id, schema, table)
Get table profile

Get column statistics for a table: row count, null counts, cardinality, and per-type details such as top values, min/max, or text lengths. Returns 404 if the table has no stored profile.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |

### Return type

[**models::TableProfileResponse**](TableProfileResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_connections

> models::ListConnectionsResponse list_connections()
List connections

List the connections in this workspace.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListConnectionsResponse**](ListConnectionsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## load_managed_table

> models::LoadManagedTableResponse load_managed_table(connection_id, schema, table, load_managed_table_request)
Load managed table from inline data, upload, or query result

Load data into a table in a `managed` connection. Give exactly one source: `data` (inline CSV, up to 2 MiB), `upload_id` (a CSV, JSON or Parquet file from `POST /v1/uploads`) or `result_id` (a saved query result). The table and its schema are created if they do not exist. `mode` sets how the rows apply.  Set `async` to run an upload or inline load as a background job; a `result_id` load always runs inline. If maintenance is running on the table, the load waits up to 60 seconds before it starts. If another write holds the table, you get 409 with `RESOURCE_LOCKED` and a `Retry-After` header.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |
**load_managed_table_request** | [**LoadManagedTableRequest**](LoadManagedTableRequest.md) |  | [required] |

### Return type

[**models::LoadManagedTableResponse**](LoadManagedTableResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_managed_table_constant_per_key

> models::ManagedTableConstantPerKeyResponse set_managed_table_constant_per_key(connection_id, schema, table, update_managed_table_request)
Declare which columns are constant per key

Set which columns never change for a given key. Keyed loads (`delete`, `update`, `upsert`) use this to search fewer files. You can change it at any time; it applies from the next load. Send an empty array to clear it.  Only declare columns that really are constant per key. If they are not, a keyed load can leave duplicate rows.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |
**update_managed_table_request** | [**UpdateManagedTableRequest**](UpdateManagedTableRequest.md) |  | [required] |

### Return type

[**models::ManagedTableConstantPerKeyResponse**](ManagedTableConstantPerKeyResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

