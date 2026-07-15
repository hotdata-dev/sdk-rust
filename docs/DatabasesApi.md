# \DatabasesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_database_schema**](DatabasesApi.md#add_database_schema) | **POST** /v1/databases/{database_id}/schemas | Add schema to database default catalog
[**add_database_table**](DatabasesApi.md#add_database_table) | **POST** /v1/databases/{database_id}/schemas/{schema}/tables | Add table to database default catalog
[**attach_database_catalog**](DatabasesApi.md#attach_database_catalog) | **POST** /v1/databases/{database_id}/catalogs | Attach catalog to database
[**create_database**](DatabasesApi.md#create_database) | **POST** /v1/databases | Create database
[**delete_database**](DatabasesApi.md#delete_database) | **DELETE** /v1/databases/{database_id} | Delete database
[**detach_database_catalog**](DatabasesApi.md#detach_database_catalog) | **DELETE** /v1/databases/{database_id}/catalogs/{connection_id} | Detach catalog from database
[**fork_database**](DatabasesApi.md#fork_database) | **POST** /v1/databases/{database_id}/fork | Fork database
[**get_database**](DatabasesApi.md#get_database) | **GET** /v1/databases/{database_id} | Get database
[**list_databases**](DatabasesApi.md#list_databases) | **GET** /v1/databases | List databases
[**load_database_table**](DatabasesApi.md#load_database_table) | **POST** /v1/databases/{database_id}/schemas/{schema}/tables/{table}/loads | Load database table from upload or query result



## add_database_schema

> models::ManagedSchemaResponse add_database_schema(database_id, add_managed_schema_request)
Add schema to database default catalog

Declare a new schema (and optionally its tables) on the database's auto-created default catalog after creation. The schema becomes reachable inside the database scope (e.g. `default.<schema>.<table>` and `information_schema.schemata`) without the caller addressing the internal default connection directly. Identifiers are normalized to lowercase.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**add_managed_schema_request** | [**AddManagedSchemaRequest**](AddManagedSchemaRequest.md) |  | [required] |

### Return type

[**models::ManagedSchemaResponse**](ManagedSchemaResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## add_database_table

> models::ManagedTableResponse add_database_table(database_id, schema, add_managed_table_request)
Add table to database default catalog

Declare a new table on an existing schema of the database's default catalog after creation. The table is added empty (declared-but-unloaded) and can be populated via the managed-table load endpoint targeting the default connection. Identifiers are normalized to lowercase.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
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


## attach_database_catalog

> attach_database_catalog(database_id, attach_database_catalog_request)
Attach catalog to database

Attach an existing connection (catalog) to a database with an optional alias. Inside the database the catalog is reachable as the alias (when set) or its original name.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**attach_database_catalog_request** | [**AttachDatabaseCatalogRequest**](AttachDatabaseCatalogRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_database

> models::CreateDatabaseResponse create_database(create_database_request)
Create database

Create a new database (a metadata-only grouping). A managed default catalog is auto-created and addressable inside the database as `default` (or the optional `default_catalog` name), with a `main` schema pre-declared so `default.main.<table>` works out of the box. The optional `name` is a free-form display label and is not required to be unique. Optional `default_catalog` overrides the name the default catalog answers to; it must be a valid SQL identifier and may not collide with the reserved catalog names `hotdata` or `information_schema`. Optional `schemas` declares additional schemas/tables on the default catalog at create time; declared tables can be loaded via the standard managed-tables-load endpoint targeting `default_connection_id`. Optional `expires_at` sets when the database expires — accepts either an RFC 3339 timestamp or a relative duration suffixed with `h` (hours), `m` (minutes), or `d` (days), e.g. `24h`, `48h`, `90m`, `7d`. When omitted, the database never expires. Expiry is best-effort: the database will not be deleted before `expires_at`, but cleanup may run later than the exact timestamp.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_database_request** | [**CreateDatabaseRequest**](CreateDatabaseRequest.md) |  | [required] |

### Return type

[**models::CreateDatabaseResponse**](CreateDatabaseResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_database

> delete_database(database_id)
Delete database

Delete a database and its auto-created default catalog. Attached catalogs are detached (their underlying connections are not deleted).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## detach_database_catalog

> detach_database_catalog(database_id, connection_id)
Detach catalog from database

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**connection_id** | **String** | Connection ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fork_database

> models::CreateDatabaseResponse fork_database(database_id, fork_database_request)
Fork database

Create a new database that is an independent fork of an existing one. The fork has its own default catalog and contains the same schemas, tables, and data as the source; the source is left unchanged. External catalogs attached to the source are re-attached to the fork. Optional `name` sets the fork's display label (defaults to the source's). Optional `expires_at` sets when the fork expires — accepts an RFC 3339 timestamp or a relative duration suffixed with `h` (hours), `m` (minutes), or `d` (days), e.g. `24h`, `90m`, `7d`. When omitted, a still-future expiry on the source is carried over; otherwise the fork never expires. Any indexes on the source's tables are not carried over.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Source database ID | [required] |
**fork_database_request** | [**ForkDatabaseRequest**](ForkDatabaseRequest.md) |  | [required] |

### Return type

[**models::CreateDatabaseResponse**](CreateDatabaseResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database

> models::DatabaseDetailResponse get_database(database_id)
Get database

Fetch a database by id. The `name` field is a display label only; it is not accepted as an identifier here.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |

### Return type

[**models::DatabaseDetailResponse**](DatabaseDetailResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_databases

> models::ListDatabasesResponse list_databases()
List databases

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListDatabasesResponse**](ListDatabasesResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## load_database_table

> models::LoadManagedTableResponse load_database_table(database_id, schema, table, load_managed_table_request)
Load database table from upload or query result

Publish data as the new contents of a table on the database's default catalog, from one of two sources — provide exactly one. The database-scoped equivalent of the connection-scoped managed-table load — addressed by `database_id`, so no `default_connection_id` is needed. With `upload_id`, a previously-uploaded file is published: CSV, JSON, and Parquet are supported; the format is auto-detected or set via `format`. With `result_id`, a persisted query result is copied into the table, so the table keeps its data even after the result expires. If the target table (or its schema) has not been declared yet, it is created automatically as part of the load — declaring tables up front is optional. `mode` selects how the data is applied: `replace` overwrites the table's contents, `append` inserts the new rows on top of the existing data. Concurrent loads against the same upload return 409. For an upload, set `async` to run the load in the background and get back a job ID to poll; add `async_after_ms` to wait briefly for it to finish before falling back to a job ID. A `result_id` load runs synchronously.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
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

