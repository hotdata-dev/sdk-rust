# QueryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | **Vec<String>** |  | 
**execution_time_ms** | **i64** |  | 
**nullable** | **Vec<bool>** | Nullable flags for each column (parallel to columns vec). True if the column allows NULL values, false if NOT NULL. | 
**preview_row_count** | **i64** | Number of rows in *this* response body. Always present. For a large result this is a bounded preview, not the grand total — see `total_row_count` and `truncated`. | 
**query_run_id** | **String** | Unique identifier for the query run record (qrun...). | 
**result_id** | Option<**String**> | Unique identifier for retrieving this result via GET /results/{id}. When non-null, the result is being persisted asynchronously. Null only when the result fit entirely in this response (`truncated: false`) but could not be persisted for later retrieval — see the `warning` field. A `truncated: true` response ALWAYS carries a non-null, resolvable `result_id` (#640 F1): a truncated result that cannot be persisted fails the request with a retryable HTTP 503 (`PERSISTENCE_UNAVAILABLE`, with a `Retry-After` header) rather than returning a partial body with a dead ticket. | [optional]
**row_count** | **i32** | **Deprecated** — use `preview_row_count` (rows in this body) and `total_row_count` (grand total) instead. Retained as a back-compat alias and always equal to `preview_row_count`; for a truncated result it is the preview count, *not* the grand total — read `total_row_count` for that. Will be removed in a future release once clients migrate. | 
**rows** | [**Vec<Vec<serde_json::Value>>**](Vec.md) | Array of rows, where each row is an array of column values. Values can be strings, numbers, booleans, or null. | 
**total_row_count** | Option<**i64**> | Grand total rows in the full result. Present (and equal to `preview_row_count`) when the whole result fit in this response; `null` while a truncated result is still being persisted. When `null`, read the authoritative total from `GET /v1/query-runs/{id}` (`row_count`) or the `X-Total-Row-Count` header on `GET /v1/results/{id}`. | [optional]
**truncated** | **bool** | True when `rows` is a bounded preview of a larger result. Fetch the full result via `result_id`. | 
**warning** | Option<**String**> | Warning message if result persistence could not be initiated. Present only when the full result is returned inline (`truncated: false`) but could not be persisted: `result_id` is then null and the result cannot be re-fetched later, though every row is in this response. A truncated result never carries a warning — if it cannot be persisted the request fails with a retryable HTTP 503 (`PERSISTENCE_UNAVAILABLE`, with a `Retry-After` header) instead (#640 F1). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


