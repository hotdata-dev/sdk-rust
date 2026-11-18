# \DatabasesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_database_schema**](DatabasesApi.md#add_database_schema) | **POST** /v1/databases/{database_id}/schemas | Add schema to database default catalog
[**add_database_table**](DatabasesApi.md#add_database_table) | **POST** /v1/databases/{database_id}/schemas/{schema}/tables | Add table to database default catalog
[**attach_database_catalog**](DatabasesApi.md#attach_database_catalog) | **POST** /v1/databases/{database_id}/catalogs | Attach catalog to database
[**bulk_create_databases**](DatabasesApi.md#bulk_create_databases) | **POST** /v1/databases/bulk | Create many databases at once
[**count_databases**](DatabasesApi.md#count_databases) | **GET** /v1/databases/count | Count databases
[**create_database**](DatabasesApi.md#create_database) | **POST** /v1/databases | Create database
[**delete_database**](DatabasesApi.md#delete_database) | **DELETE** /v1/databases/{database_id} | Delete database
[**delete_database_batch**](DatabasesApi.md#delete_database_batch) | **DELETE** /v1/databases/bulk/{batch_id} | Delete a database batch
[**detach_database_catalog**](DatabasesApi.md#detach_database_catalog) | **DELETE** /v1/databases/{database_id}/catalogs/{connection_id} | Detach catalog from database
[**fork_database**](DatabasesApi.md#fork_database) | **POST** /v1/databases/{database_id}/fork | Fork database
[**get_database**](DatabasesApi.md#get_database) | **GET** /v1/databases/{database_id} | Get database
[**get_database_batch**](DatabasesApi.md#get_database_batch) | **GET** /v1/databases/bulk/{batch_id} | Get a database batch
[**get_database_lineage**](DatabasesApi.md#get_database_lineage) | **GET** /v1/databases/{database_id}/lineage | Get database lineage
[**list_databases**](DatabasesApi.md#list_databases) | **GET** /v1/databases | List databases
[**load_database_table**](DatabasesApi.md#load_database_table) | **POST** /v1/databases/{database_id}/schemas/{schema}/tables/{table}/loads | Load database table from inline data, upload, or query result
[**lookup_database_by_name**](DatabasesApi.md#lookup_database_by_name) | **GET** /v1/databases/by-name | Look up a database by name
[**set_database_table_constant_per_key**](DatabasesApi.md#set_database_table_constant_per_key) | **PUT** /v1/databases/{database_id}/schemas/{schema}/tables/{table}/constant-per-key | Declare which columns are constant per key



## add_database_schema

> models::ManagedSchemaResponse add_database_schema(database_id, add_managed_schema_request)
Add schema to database default catalog

Add a schema, and optionally its tables, to the database's `default` catalog. Names are stored in lowercase.

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

Add an empty table to a schema in the database's `default` catalog. Names are stored in lowercase. You do not have to declare tables first: a load creates the table if it is missing.

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

Attach a catalog so its tables can be queried in this database. To query across two databases, pass the other database's `default_connection_id` as `connection_id`.  The catalog is read-only here and nothing is copied. It answers to `alias`, or to its own name if you omit it. That name cannot be `default` or a reserved name, so attaching another database's `default` catalog needs an `alias`. Attaching is not transitive: you do not see the attached catalog's own attachments.

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


## bulk_create_databases

> models::DatabaseBatchResponse bulk_create_databases(bulk_create_databases_request)
Create many databases at once

Create many databases from one template, in the background. The response returns at once with a `batch_id` and a job to poll.  Pass `schemas` to declare the same tables in every database. List the databases a batch created with `GET /v1/databases?batch=<batch_id>`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bulk_create_databases_request** | [**BulkCreateDatabasesRequest**](BulkCreateDatabasesRequest.md) |  | [required] |

### Return type

[**models::DatabaseBatchResponse**](DatabaseBatchResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## count_databases

> models::DatabaseCountResponse count_databases(search, batch)
Count databases

Count the databases in the workspace. `search` and `batch` filter the same way as on `GET /v1/databases`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search** | Option<**String**> | Count only databases whose name contains this text, ignoring case. |  |
**batch** | Option<**String**> | Count only databases from this bulk-create batch. |  |

### Return type

[**models::DatabaseCountResponse**](DatabaseCountResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_database

> models::CreateDatabaseResponse create_database(create_database_request)
Create database

Create a database. Each database gets its own `default` catalog with a `main` schema, so `default.main.<table>` works right away. You can declare more schemas and tables in `schemas`.  `name` is a label and does not have to be unique. Set `if_not_exists` to get back the database that already has this `name` (status 200) instead of creating a new one; the rest of the request is then ignored. Set `expires_at` to have the database deleted after that time. Deletion never happens earlier, but can happen later.

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

Delete a database and its `default` catalog. Catalogs attached to it are detached, not deleted.  Returns 409 while another database has this database's catalog attached; detach it there first. After `expires_at` the database is deleted anyway, and the other database loses access.

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


## delete_database_batch

> models::DeleteDatabaseBatchResponse delete_database_batch(batch_id)
Delete a database batch

Stop a batch and delete the databases it created, then the batch.  This works only while no database in the batch holds loaded data. Declared but unloaded tables are fine. If any database has data, the request returns 409; delete those databases one at a time.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_id** | **String** | Batch ID | [required] |

### Return type

[**models::DeleteDatabaseBatchResponse**](DeleteDatabaseBatchResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## detach_database_catalog

> detach_database_catalog(database_id, connection_id)
Detach catalog from database

Detach a catalog from a database. The catalog and its data are not deleted.

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

Create a new database that starts as a copy of an existing one: same schemas, tables, data and attached catalogs. The two are independent after that; the source does not change.  Indexes come across and work at once, except vector indexes that use the `ivf` algorithm and indexes not up to date with the table's latest load. Create those again on the fork. A fork shares the source's storage at first, so it adds no stored bytes until it is written to or maintenance rewrites a table.

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

Get a database by ID. To find one by name, use `GET /v1/databases/by-name`.

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


## get_database_batch

> models::DatabaseBatchResponse get_database_batch(batch_id)
Get a database batch

Get a batch: how many databases were requested and how many exist so far. Poll this to follow progress.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_id** | **String** | Batch ID | [required] |

### Return type

[**models::DatabaseBatchResponse**](DatabaseBatchResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_lineage

> models::DatabaseLineageResponse get_database_lineage(database_id, forks_limit)
Get database lineage

Show where a database was forked from and what was forked from it.  `ancestors` lists the fork chain up to the original, nearest first. `forks` lists databases forked directly from this one, newest first; `fork_count` is the full total when the list is cut short. Deleted databases stay in the record with `exists` set to false. A database with no forks returns empty lists and itself as `root_id`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**forks_limit** | Option<**i32**> | Number of direct forks to list, 1 to 100. Defaults to 25. |  |

### Return type

[**models::DatabaseLineageResponse**](DatabaseLineageResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_databases

> models::ListDatabasesResponse list_databases(limit, cursor, search, batch)
List databases

List databases, newest first, one page at a time. If `has_more` is true, pass `next_cursor` as `cursor` to get the next page.  `search` matches names that contain the text. To get the one database with an exact name, use `GET /v1/databases/by-name`. `batch` lists only the databases a bulk create made.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Databases per page, 1 to 100. Defaults to 100. |  |
**cursor** | Option<**String**> | `next_cursor` from the previous page. |  |
**search** | Option<**String**> | Return only databases whose name contains this text, ignoring case. |  |
**batch** | Option<**String**> | Return only databases from this bulk-create batch, ordered by ID. |  |

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
Load database table from inline data, upload, or query result

Load data into a table in the database's `default` catalog. Give exactly one source: `data` (inline CSV, up to 2 MiB), `upload_id` (a CSV, JSON or Parquet file from `POST /v1/uploads`) or `result_id` (a saved query result). The table and its schema are created if they do not exist. `mode` sets how the rows apply.  Set `async` to run an upload or inline load as a background job; a `result_id` load always runs inline. If maintenance is running on the table, the load waits up to 60 seconds before it starts. If another write holds the table, you get 409 with `RESOURCE_LOCKED` and a `Retry-After` header.

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


## lookup_database_by_name

> models::DatabaseDetailResponse lookup_database_by_name(name)
Look up a database by name

Get the database with this exact name. Matching ignores case only for unaccented Latin letters; all other characters must match exactly.  Returns 409 if more than one database has the name. Use the database ID for those.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Full name of the database. Case is ignored for ASCII letters only. | [required] |

### Return type

[**models::DatabaseDetailResponse**](DatabaseDetailResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_database_table_constant_per_key

> models::ManagedTableConstantPerKeyResponse set_database_table_constant_per_key(database_id, schema, table, update_managed_table_request)
Declare which columns are constant per key

Set which columns never change for a given key. Keyed loads (`delete`, `update`, `upsert`) use this to search fewer files. You can change it at any time; it applies from the next load. Send an empty array to clear it.  Only declare columns that really are constant per key. If they are not, a keyed load can leave duplicate rows.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
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

