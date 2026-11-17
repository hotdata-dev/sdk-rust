# \StreamsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_stream_handler**](StreamsApi.md#create_stream_handler) | **POST** /v1/databases/{database_id}/streams | Create a stream
[**delete_stream_handler**](StreamsApi.md#delete_stream_handler) | **DELETE** /v1/streams/{stream_id} | Delete a stream
[**get_stream_handler**](StreamsApi.md#get_stream_handler) | **GET** /v1/streams/{stream_id} | Get a stream
[**ingest_stream_handler**](StreamsApi.md#ingest_stream_handler) | **POST** /v1/streams/{stream_id}/spans/ingest/sync | Append rows to a stream



## create_stream_handler

> create_stream_handler(database_id, create_stream_request)
Create a stream

Create a stream that writes one table, which callers then post span or log rows to without declaring its columns. The stream's first request decides the columns: every key in its rows becomes a column, with a type taken from the values, and string columns that hold large text are stored apart, so queries that do not read them stay fast. Each key that the first rows show inside an object field also gets a column of its own, named `field.key`: a query of `field['key']` reads that column instead of the whole object, and gives the same rows. Name a column in `columns` to fix its type or role before that request, or to add a column the first rows lack. The table is created now and gets its columns then. It is partitioned by the day of its time column. With no `time_column`, the server adds the column `_ingested_at` and fills it with the time each request is received. Returns the stream with status `waiting_for_data` until that request arrives. Available only where it has been enabled.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**create_stream_request** | [**CreateStreamRequest**](CreateStreamRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_stream_handler

> delete_stream_handler(stream_id)
Delete a stream

Delete a stream, its table and every row in it. Rows the stream accepted that cannot be queried yet are discarded too. From the moment the delete begins, requests to the stream answer 404. Once it returns, the table's name is free for a new table or stream. A delete that fails can be retried.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stream_id** | **String** | Stream ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_stream_handler

> get_stream_handler(stream_id)
Get a stream

A stream's table, status, time column and column hints, and once its first request has arrived, each column with its type and role: `time`, `large_text`, `normal`, `map_key` or `extra`. A `map_key` column holds one key of an object field, named in its `from` as `map` and `key`: in every row it equals that field's value for the key, or null when the row lacks it. The `extra` column, `_extra`, keeps by key, as JSON text, what a row sent that its columns cannot hold: a field that is not a column, or a value of the wrong type for its column, which then reads null. `rows` is how many rows the table holds, and `last_flushed_at` when rows were last stored, or null before the first. Rows are accepted first and stored within seconds: `pending_bytes` is how many bytes of accepted rows the server that answers holds and has not stored yet, and `oldest_pending_age_ms` how long the oldest of them has waited. A server that holds none of the stream's rows returns null for both.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stream_id** | **String** | Stream ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_stream_handler

> ingest_stream_handler(stream_id, body)
Append rows to a stream

Append a batch of rows to a stream's table, and return once they are stored. The body is a JSON object whose `spans` field is an array of rows, up to 64 MiB. The stream's first request decides its columns and returns them. After that, each row's fields are matched to the columns by name: a field the table does not have is ignored, and a column a row leaves out is null. A `map_key` column is filled from its object field, never from a row field of the same name. Every row must carry the stream's time column. Rows can be queried within seconds.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stream_id** | **String** | Stream ID | [required] |
**body** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

