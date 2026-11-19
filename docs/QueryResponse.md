# QueryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | **Vec<String>** | Column names, in order. | 
**execution_time_ms** | **i64** | Time the query took to run, in milliseconds. | 
**nullable** | **Vec<bool>** | Whether each column, in the same order as `columns`, can hold nulls. | 
**preview_row_count** | **i64** | Number of rows in this response. | 
**query_run_id** | **String** | ID of the query run. | 
**result_id** | Option<**String**> | ID to fetch this result again with `GET /v1/results/{id}`. Null only when every row is in this response but the result could not be saved; see `warning`. | [optional]
**rollup** | Option<[**models::RollupUsage**](RollupUsage.md)> |  | [optional]
**row_count** | **i32** | Deprecated: use `preview_row_count` or `total_row_count`. Always equal to `preview_row_count`. | 
**rows** | [**Vec<Vec<models::JsonCell>>**](Vec.md) | Result rows. Each row is an array of values in column order. | 
**total_row_count** | Option<**i64**> | Number of rows in the full result. Null while a truncated result is still being saved; read `row_count` from the query run once it succeeds. | [optional]
**truncated** | **bool** | True when `rows` is only a preview of a larger result. Fetch the full result with `result_id`. | 
**warning** | Option<**String**> | Why the result could not be saved. Present only when `result_id` is null; every row is still in this response. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


