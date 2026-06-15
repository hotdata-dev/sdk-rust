# QueryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | **Vec<String>** |  | 
**execution_time_ms** | **i64** |  | 
**nullable** | **Vec<bool>** | Nullable flags for each column (parallel to columns vec). True if the column allows NULL values, false if NOT NULL. | 
**preview_row_count** | **i64** | Number of rows in *this* response body (`rows.len()`). Always present. For a large result this is a bounded preview, not the grand total — see `total_row_count` and `truncated` (#640). | 
**query_run_id** | **String** | Unique identifier for the query run record (qrun...). | 
**result_id** | Option<**String**> | Unique identifier for retrieving this result via GET /results/{id}. Null if catalog registration failed (see `warning` field for details). When non-null, the result is being persisted asynchronously. | [optional]
**row_count** | **i32** | **Deprecated** — use `preview_row_count` (rows in this body) and `total_row_count` (grand total) instead. Retained for backward compatibility and currently always equal to `preview_row_count`; it will be removed in a future release once clients migrate to the count fields below (#640). | 
**rows** | [**Vec<Vec<serde_json::Value>>**](Vec.md) | Array of rows, where each row is an array of column values. Values can be strings, numbers, booleans, or null. | 
**total_row_count** | Option<**i64**> | Grand total rows in the full result. Present (and equal to `preview_row_count`) when the whole result fit in this response; `null` while a truncated result is still being persisted. When `null`, read the authoritative total from `GET /v1/query-runs/{id}` (`row_count`) or the `X-Total-Row-Count` header on `GET /v1/results/{id}` (#640). | [optional]
**truncated** | **bool** | True when `rows` is a bounded preview of a larger result. Fetch the full result via `result_id` (#640). Always `false` until bounded streaming is enabled; clients should still branch on it so no code change is needed when truncation goes live. | 
**warning** | Option<**String**> | Warning message if result persistence could not be initiated. When present, `result_id` will be null and the result cannot be retrieved later. The query results are still returned in this response. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


