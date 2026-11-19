# \RollupsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**declare_rollup**](RollupsApi.md#declare_rollup) | **POST** /v1/databases/{database_id}/rollups | Declare a rollup on a table
[**drop_rollup**](RollupsApi.md#drop_rollup) | **DELETE** /v1/databases/{database_id}/rollups/{rollup_id} | Drop a rollup
[**get_rollup**](RollupsApi.md#get_rollup) | **GET** /v1/databases/{database_id}/rollups/{rollup_id} | Get a rollup
[**list_rollups**](RollupsApi.md#list_rollups) | **GET** /v1/databases/{database_id}/rollups | List a database's rollups
[**rebuild_rollup**](RollupsApi.md#rebuild_rollup) | **POST** /v1/databases/{database_id}/rollups/{rollup_id}/rebuild | Rebuild a rollup
[**set_rollup_window**](RollupsApi.md#set_rollup_window) | **PUT** /v1/databases/{database_id}/rollups/{rollup_id}/window | Set a rollup's window



## declare_rollup

> models::RollupBuildResponse declare_rollup(database_id, declare_rollup_request)
Declare a rollup on a table

Declare a rollup: pre-aggregated rows of one table, one per time bucket and set of dimension values, which answer matching aggregate queries without reading the whole table. Answers are the same as reading the table. The table must hold data, and its time column must be a timestamp with no time zone or in UTC.  The rollup starts as `building` and becomes `ready` when its build catches up with the table; poll the returned job or the rollup. When `job_id` is null the build could not be started: rebuild the rollup to start it.  A query uses a ready rollup when it aggregates with `count`, `sum`, `avg`, `min`, `max`, or `approx_percentile_cont` / `approx_median` at the declared `centroids` (approximate answers only); groups by the time column in buckets that are a multiple of `bucket_width_secs`, and by dimensions; filters only on dimensions and the time column; and, when the rollup has a `filter`, contains each of its `AND`-joined parts written the same way. `min` and `max` of a floating-point column are always read from the table. The query's response says whether a rollup was used.  Declaring a rollup changes how the table can be written. Rows whose time is older than `window_secs` can no longer be changed or deleted, and SQL `DELETE` and `UPDATE` on the table are refused: change rows with a keyed load whose key includes the time column. Appends are always allowed, and so is replacing the table, after which its rollups need a rebuild and queries read the table until then. A fork of the database does not carry rollups, and dropping the table drops them. A rollup cannot be changed: drop it and declare a new one, which builds from the start.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**declare_rollup_request** | [**DeclareRollupRequest**](DeclareRollupRequest.md) |  | [required] |

### Return type

[**models::RollupBuildResponse**](RollupBuildResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## drop_rollup

> drop_rollup(database_id, rollup_id)
Drop a rollup

Drop a rollup and its rows. The table does not change. Returns once the rollup is gone, which can take up to two minutes when a build step is running on it. When the last rollup on a table is dropped, the table's write window no longer applies.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**rollup_id** | **String** | Rollup ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_rollup

> models::Rollup get_rollup(database_id, rollup_id)
Get a rollup

Get a rollup: its definition, its status, and how far its build has come.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**rollup_id** | **String** | Rollup ID | [required] |

### Return type

[**models::Rollup**](Rollup.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_rollups

> models::ListRollupsResponse list_rollups(database_id, schema_name, table_name)
List a database's rollups

List the rollups on a database's tables. Filter by schema, or by schema and table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**schema_name** | Option<**String**> | Only rollups on tables of this schema |  |
**table_name** | Option<**String**> | Only rollups on this table; requires `schema_name` |  |

### Return type

[**models::ListRollupsResponse**](ListRollupsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## rebuild_rollup

> models::RollupBuildResponse rebuild_rollup(database_id, rollup_id)
Rebuild a rollup

Start building a rollup again, in the background, and return the build job. A `needs_rebuild` rollup (left that way when its table is replaced) is emptied and rebuilt; queries read the table until the build finishes. A `build_failed` rollup, or a `building` one whose build is not running, resumes where it stopped.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**rollup_id** | **String** | Rollup ID | [required] |

### Return type

[**models::RollupBuildResponse**](RollupBuildResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_rollup_window

> models::Rollup set_rollup_window(database_id, rollup_id, set_rollup_window_request)
Set a rollup's window

Set how long after a row's time it can still be changed or deleted. This applies to every rollup on the same table. Appends are always allowed. Nothing is rebuilt: a wider window applies as time passes, and a narrower one at the next refresh.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**rollup_id** | **String** | Rollup ID | [required] |
**set_rollup_window_request** | [**SetRollupWindowRequest**](SetRollupWindowRequest.md) |  | [required] |

### Return type

[**models::Rollup**](Rollup.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

