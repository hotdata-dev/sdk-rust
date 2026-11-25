# StreamResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**column_hints** | Option<[**Vec<models::StreamColumnHint>**](StreamColumnHint.md)> | Columns the stream was created with. Omitted when there are none. | [optional]
**columns** | Option<[**Vec<models::StreamColumn>**](StreamColumn.md)> | The stream's columns, in table order. Omitted until the first request sets them. | [optional]
**id** | **String** | Stream ID. | 
**id_index_up_to_date** | Option<**bool**> | True when the ID index has recorded every stored row for each ID column the server indexes (trace and span IDs, unless configured otherwise). False while it is catching up, or when a column was added to the index after rows were stored. It reports what the index recorded and does not run a lookup. Lookups by an ID column it does not cover return the same rows, more slowly. Omitted when the server does not index IDs. | [optional]
**last_flushed_at** | Option<**String**> | When rows were last stored. Null before the first time. | [optional]
**oldest_pending_age_ms** | Option<**i64**> | How long the oldest of those rows has waited, in milliseconds. Null when none waits. | [optional]
**pending_bytes** | Option<**i64**> | Bytes of accepted rows that the answering server has not stored yet. Null when it holds none. | [optional]
**retention_days** | Option<**i32**> | Days of rows to keep. Omitted when none was set. | [optional]
**rows** | **i64** | Rows stored in the table. | 
**schema** | **String** | Schema of the stream's table. | 
**status** | **String** | `waiting_for_data` until the first request, then `active`. | 
**table** | **String** | Table the stream writes to. | 
**time_column** | Option<**String**> | Row field that holds each row's time. Null when Hotdata sets the time in `_ingested_at`. | [optional]
**time_unit** | Option<**String**> | Format of `time_column`: `ns`, `us`, `ms`, `s` or `rfc3339`. Null when `time_column` is null. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


