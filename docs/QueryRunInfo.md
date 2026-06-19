# QueryRunInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bytes_scanned** | Option<**i64**> | Total bytes of table data read from storage to run this query. `null` when the query touches no table at all (for example a constant expression like `SELECT 1`). May be `0` when the query reads a table but not its row data — for example a row count served from table statistics. | [optional]
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
**user_public_id** | Option<**String**> | Caller identity derived from the Authorization Bearer token (SHA-256 hash). Format: `user_{first_10_hex_chars}`. | [optional]
**warning_message** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


