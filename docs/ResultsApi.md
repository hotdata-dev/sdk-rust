# \ResultsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_result**](ResultsApi.md#get_result) | **GET** /v1/results/{id} | Get result
[**list_results**](ResultsApi.md#list_results) | **GET** /v1/results | List results



## get_result

> models::GetResultResponse get_result(id, x_database_id, offset, limit, format)
Get result

Retrieve a persisted query result by ID. The response format for the `ready` state is selected by `Accept` header or `?format=` query param; non-ready states use the same status codes and JSON body shape regardless of format.  | Result status         | Status × body                                                                | |-----------------------|------------------------------------------------------------------------------| | `ready` + JSON        | 200 `application/json` — `GetResultResponse` with `columns`, `rows`, etc.    | | `ready` + Arrow       | 200 `application/vnd.apache.arrow.stream` — schema, RecordBatches, EOS       | | `ready` + CSV         | 200 `text/csv; charset=utf-8` — single header row, streamed batch-by-batch   | | `ready` + Markdown    | 200 `text/markdown; charset=utf-8` — GitHub-flavored pipe table, streamed   | | `ready` + Parquet     | 200 `application/vnd.apache.parquet` — raw parquet bytes (no conversion)     | | `pending`/`processing`| 202 `application/json` `{status, result_id}` + `Retry-After`                 | | `failed`              | 409 `application/json` `{status, result_id, error_message}`                  | | not found             | 404 `application/json` (`ApiErrorResponse`)                                  |  `?format=` accepts `arrow`, `json`, `csv`, `md`, `parquet` and takes precedence over `Accept`. `markdown` is accepted as a runtime alias for `md`. Use `?offset=N&limit=M` to slice the result; `offset` defaults to 0 and `limit` is unbounded by default. Both must be non-negative; invalid values return 400. When a finite `limit` doesn't reach the end of the result, a `Link` header with `rel=\"next\"` points at the following page. `?offset`/`?limit` are ignored for `format=parquet` since that path returns the underlying file unchanged.  JSON is the only format with a size limit. Building a JSON body requires holding the whole slice in memory at once, so a request whose JSON body would exceed the instance's per-fetch memory budget is refused with 413 and the error code `RESULT_TOO_LARGE`, and a request that would push the total across concurrent JSON fetches over that budget is refused with 429. If the result's size cannot be determined at all, the fetch is refused with 503 rather than served unchecked. None of the three returns partial data. To read a result of any size, request it as `?format=arrow`, `csv`, `md`, or `parquet` — those stream and are never size-limited. Paging with `offset`/`limit` is not an alternative: the rows come back in no guaranteed order, so pages are not a reliable way to read one result in parts.  Ready responses (Arrow, CSV, Markdown, JSON) carry `X-Total-Row-Count` (the full result row count, independent of offset/limit); the JSON body repeats it as `total_row_count`, so a client that cannot read response headers can still compare it against `row_count` to tell a complete result from a windowed one. Responses are streamed end-to-end, so a client can disconnect at any time and the server stops reading.  IEEE special floats (`±Inf`, `NaN`) have no canonical JSON representation. For cross-format consistency the JSON, CSV, and Markdown paths emit them as `null` / empty cells, and JSON `nullable[]` is widened to match. The Arrow IPC and Parquet bodies are binary round-trip formats and preserve the raw IEEE values; callers cross-checking a result across CSV and Parquet should not byte-compare those slots.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Result ID | [required] |
**x_database_id** | **String** | Database the result belongs to (required) | [required] |
**offset** | Option<**i32**> | Rows to skip (default: 0) |  |
**limit** | Option<**i32**> | Maximum rows to return (default: unbounded) |  |
**format** | Option<[**ResultsFormatQuery**](ResultsFormatQuery.md)> | `arrow`, `json`, `csv`, `md`, or `parquet` — overrides the `Accept` header. `markdown` is also accepted at runtime as an alias for `md`. |  |

### Return type

[**models::GetResultResponse**](GetResultResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/vnd.apache.arrow.stream, text/csv, text/markdown, application/vnd.apache.parquet

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_results

> models::ListResultsResponse list_results(x_database_id, limit, offset)
List results

List stored results for the database named by the required X-Database-Id header.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**x_database_id** | **String** | Database to scope the results to (required) | [required] |
**limit** | Option<**i32**> | Maximum number of results (default: 100, max: 1000) |  |
**offset** | Option<**i32**> | Pagination offset (default: 0) |  |

### Return type

[**models::ListResultsResponse**](ListResultsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

