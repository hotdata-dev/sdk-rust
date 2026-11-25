# \StreamsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_stream_handler**](StreamsApi.md#create_stream_handler) | **POST** /v1/databases/{database_id}/streams | Create a stream
[**delete_stream_handler**](StreamsApi.md#delete_stream_handler) | **DELETE** /v1/streams/{stream_id} | Delete a stream
[**get_stream_handler**](StreamsApi.md#get_stream_handler) | **GET** /v1/streams/{stream_id} | Get a stream
[**ingest_stream_handler**](StreamsApi.md#ingest_stream_handler) | **POST** /v1/streams/{stream_id}/spans/ingest/sync | Append rows to a stream



## create_stream_handler

> models::StreamResponse create_stream_handler(database_id, create_stream_request)
Create a stream

Create a stream: a table you post span or log rows to without declaring its columns. The first request sets the columns from its rows. Each key becomes a column typed from its values, and each key inside an object field also gets its own `field.key` column. Use `columns` to set a column's type or role in advance.  The table is partitioned by day of its time column. With no `time_column`, Hotdata adds `_ingested_at` and sets it to the time each request arrives. The stream's status is `waiting_for_data` until the first request. Streams are available only where they are enabled.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**create_stream_request** | [**CreateStreamRequest**](CreateStreamRequest.md) |  | [required] |

### Return type

[**models::StreamResponse**](StreamResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_stream_handler

> delete_stream_handler(stream_id)
Delete a stream

Delete a stream, its table and all its rows, including rows not stored yet. Requests to the stream return 404 as soon as the delete starts. When it finishes, the table name can be used again. If the delete fails, retry it.

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

> models::StreamResponse get_stream_handler(stream_id)
Get a stream

Get a stream: its table, status, time column and column hints. After the first request, it also lists each column with its type and role: `time`, `large_text`, `normal`, `map_key` or `extra`.  A `map_key` column holds one key of an object field; `from` names the field and key. The `_extra` column keeps, as JSON text, any field that has no column and any value of the wrong type for its column. `pending_bytes` and `oldest_pending_age_ms` describe accepted rows that the answering server has not stored yet; both are null when it holds none. `id_index_up_to_date` says whether every stored row is indexed for each ID column the server indexes; it is omitted when the server does not index IDs.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stream_id** | **String** | Stream ID | [required] |

### Return type

[**models::StreamResponse**](StreamResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_stream_handler

> models::IngestStreamResponse ingest_stream_handler(stream_id, ingest_stream_request)
Append rows to a stream

Append rows to a stream. The body is a JSON object with a `spans` array of rows, up to 64 MiB. The first request sets the stream's columns and returns them.  After that, fields match columns by name. A missing field is null. A field with no column, or a value of the wrong type, goes to the `_extra` column. Every row must carry the time column, unless Hotdata sets `_ingested_at`. The call returns once the rows are accepted; they can be queried within seconds.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stream_id** | **String** | Stream ID | [required] |
**ingest_stream_request** | [**IngestStreamRequest**](IngestStreamRequest.md) |  | [required] |

### Return type

[**models::IngestStreamResponse**](IngestStreamResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

