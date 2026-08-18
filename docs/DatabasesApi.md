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
[**list_databases**](DatabasesApi.md#list_databases) | **GET** /v1/databases | List databases
[**load_database_table**](DatabasesApi.md#load_database_table) | **POST** /v1/databases/{database_id}/schemas/{schema}/tables/{table}/loads | Load database table from inline data, upload, or query result



## add_database_schema

> models::ManagedSchemaResponse add_database_schema(database_id, add_managed_schema_request)
Add schema to database default catalog

Declare a new schema (and optionally its tables) on the database's auto-created default catalog after creation. The schema becomes reachable inside the database scope (e.g. `default.<schema>.<table>` and `information_schema.schemata`) without the caller naming the database's default connection. Identifiers are normalized to lowercase.

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


## bulk_create_databases

> models::DatabaseBatchResponse bulk_create_databases(bulk_create_databases_request)
Create many databases at once

Create many databases from one template in a single request. The databases are created in the background: the response returns immediately with a batch and a job to poll.  The databases are not returned inline. List the ones a batch created with `GET /databases?batch=<batch_id>`; they also appear in the normal database listing alongside every other database.  Each database gets a default catalog and schema. Declare tables on all of them by passing `schemas`, in the same shape a single create accepts — a batch of 10,000 declaring one table yields 10,000 databases that each hold that table and are ready to load, with no follow-up call per database. Omit `schemas` and the databases are created empty. Either way, load data into them exactly as you would a database created individually.

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

Return the total number of databases in the workspace. This is the whole-workspace total, not a page size: the `count` field on the listing reports how many rows that one page returned, so totalling a workspace from `GET /v1/databases` means walking every page. Pass `search` to count only databases whose name contains that text (case-insensitive), or `batch` with the `batch_id` returned by a bulk-creation call to count only that batch's databases. The filters mean exactly what they mean on the listing, so a count and a listing given the same filters describe the same set.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search** | Option<**String**> | Case-insensitive substring filter on the database name. When set, only databases whose name contains this text are counted. |  |
**batch** | Option<**String**> | Count only the databases belonging to one bulk-creation batch, identified by the `batch_id` that call returned. |  |

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


## delete_database_batch

> models::DeleteDatabaseBatchResponse delete_database_batch(batch_id)
Delete a database batch

Stop a batch that is still filling and delete the databases it created, then the batch itself.  Only batches whose databases hold no data can be removed this way. Tables that were declared but never loaded do not prevent it, so a batch created with `schemas` stays deletable. If any database in the batch has had data loaded into it, the request is rejected and those databases must be deleted one at a time — removing a database that holds data is per-database work that cannot be batched.

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


## get_database_batch

> models::DatabaseBatchResponse get_database_batch(batch_id)
Get a database batch

Fetch a batch by id: how many databases were requested and how many exist so far. Poll this to follow progress.

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


## list_databases

> models::ListDatabasesResponse list_databases(limit, cursor, search, batch)
List databases

List databases in the workspace, newest first, one page at a time. When no `limit` is given a default page size is applied, so a single call returns at most one page rather than every database. If the response's `has_more` is true, pass its `next_cursor` value back as the `cursor` query parameter to fetch the next page. Pass `search` to return only databases whose name contains that text (case-insensitive). Pass `batch` with the `batch_id` returned by a bulk-creation call to list only that batch's databases.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of databases to return in this page (1–100). Values outside the range are clamped. |  |
**cursor** | Option<**String**> | Opaque pagination cursor from a previous response's `next_cursor`. |  |
**search** | Option<**String**> | Case-insensitive substring filter on the database name. When set, only databases whose name contains this text are returned; paging and newest-first ordering are unchanged. |  |
**batch** | Option<**String**> | List only the databases belonging to one bulk-creation batch, identified by the `batch_id` that call returned.  Bulk-created databases also appear in the unfiltered listing alongside every other database; this narrows the listing to one batch. Paging works the same way, but results are ordered by database id rather than newest-first, because every database in a batch is created at once. |  |

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

Publish data as the new contents of a table on the database's default catalog, from one of three sources — provide exactly one. The database-scoped equivalent of the connection-scoped managed-table load — addressed by `database_id`, so no `default_connection_id` is needed. With `data`, CSV text is sent inline in this request, up to 2 MiB; column types are detected from the data unless `columns` declares them, and a larger payload is rejected with 413 and the error code `INLINE_DATA_TOO_LARGE`, at which point the data should be uploaded and loaded by `upload_id` instead. With `upload_id`, a previously-uploaded file is published: CSV, JSON, and Parquet are supported; the format is auto-detected or set via `format`. With `result_id`, a persisted query result is copied into the table, so the table keeps its data even after the result expires. If the target table (or its schema) has not been declared yet, it is created automatically as part of the load — declaring tables up front is optional. `mode` selects how the data is applied: `replace` overwrites the table's contents, `append` inserts the new rows on top of the existing data. Concurrent loads against the same upload return 409. For an upload or inline data, set `async` to run the load in the background and get back a job ID to poll; add `async_after_ms` to wait briefly for it to finish before falling back to a job ID. A `result_id` load runs synchronously.

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

