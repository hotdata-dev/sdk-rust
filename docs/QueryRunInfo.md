# QueryRunInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bytes_scanned** | Option<**i64**> | Bytes the query read from storage. Cached data is not counted, so a repeated query often reports `0`; null when the query read no table. | [optional]
**completed_at** | Option<**String**> |  | [optional]
**created_at** | **String** |  | 
**error_message** | Option<**String**> | Why the run failed. | [optional]
**execution_time_ms** | Option<**i64**> |  | [optional]
**id** | **String** |  | 
**result_id** | Option<**String**> | ID of the saved result, ready to fetch once `status` is `succeeded`. Absent when no result was saved; `warning_message` then says why. | [optional]
**row_count** | Option<**i64**> | Number of rows in the full result. | [optional]
**rows_scanned** | Option<**i64**> | Rows the query read, before filtering or aggregation. Null when the query read no table. | [optional]
**saved_query_id** | Option<**String**> | ID of the saved query this run executed, if any. | [optional]
**saved_query_version** | Option<**i32**> | Version of the saved query this run executed, if any. | [optional]
**server_processing_ms** | Option<**i64**> | Time from query start until the result was ready, in milliseconds. Excludes network time. | [optional]
**snapshot_id** | **String** |  | 
**sql_hash** | **String** |  | 
**sql_text** | **String** |  | 
**status** | **String** | `running`, `succeeded`, `failed` or `interrupted`. An interrupted run stopped before finishing and is safe to retry. | 
**trace_id** | Option<**String**> |  | [optional]
**user_public_id** | Option<**String**> | Account that ran the query, taken from the access token. A credential with no account gets a stable `user_`-prefixed ID instead. | [optional]
**warning_message** | Option<**String**> | A problem that did not fail the run, such as a result that could not be saved. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


