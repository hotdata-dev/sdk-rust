# QueryRunInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bytes_scanned** | Option<**i64**> | Bytes this query actually fetched from storage. Not a measure of how much data the query covered — it counts the reads that reached storage, so the same SQL over the same rows reports a different number depending on what was already cached.  `null` means the query touched no table at all (for example a constant expression like `SELECT 1`).  `0` means the query read a table but fetched nothing from storage. It is a normal, common answer, not an error or a missing measurement, and it does not mean the query did no work — `rows_scanned` shows the rows it went through. The usual cause is a warm cache: re-running a query whose data or file metadata is already held in memory reports far fewer bytes than the first run, frequently `0`, with `rows_scanned` unchanged. A query answered from table statistics alone (a row count, say) also reports `0`.  Because of this, `bytes_scanned` is not a proxy for query cost or query volume. `GET /v1/usage` sums this field over a period, so that total is the storage read a workspace caused, not the work its queries did: repeating one query cheaply adds little or nothing to it. | [optional]
**completed_at** | Option<**String**> |  | [optional]
**created_at** | **String** |  | 
**error_message** | Option<**String**> |  | [optional]
**execution_time_ms** | Option<**i64**> |  | [optional]
**id** | **String** |  | 
**result_id** | Option<**String**> |  | [optional]
**row_count** | Option<**i64**> |  | [optional]
**rows_scanned** | Option<**i64**> | Total rows read from storage to run this query, before any filtering or aggregation. Distinct from `row_count`, which is how many rows the query returned. `null` when the query reads no table data from storage. | [optional]
**saved_query_id** | Option<**String**> |  | [optional]
**saved_query_version** | Option<**i32**> |  | [optional]
**server_processing_ms** | Option<**i64**> | Total server-side processing time for this query (milliseconds). Measured from query start to result ready. Includes SQL execution, task spawning, and result preparation. Does not include network transit. Populated for all completed query runs (sync and async). | [optional]
**snapshot_id** | **String** |  | 
**sql_hash** | **String** |  | 
**sql_text** | **String** |  | 
**status** | **String** |  | 
**trace_id** | Option<**String**> |  | [optional]
**user_public_id** | Option<**String**> | Who ran this query: the account id from the access token the request was made with. Use it to group a caller's query history.  Requests made with a credential that identifies no account instead record an opaque `user_`-prefixed identifier, which is stable for that credential but cannot be resolved to an account. | [optional]
**warning_message** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


