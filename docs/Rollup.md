# Rollup

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bucket_width_secs** | **i64** | Width of each time bucket, in seconds. | 
**covered_until** | Option<**String**> | End (exclusive) of the time range the rollup has built; null until the build's first step. Build progress, not freshness: queries also read the table for rows the rollup has not absorbed, so answers are the same either way. It advances while `building`, and after the table is replaced it describes the old table until the rollup is rebuilt. | [optional]
**created_at** | **String** | When the rollup was declared. | 
**database_id** | **String** | Database that holds the table. | 
**dimensions** | **Vec<String>** | Columns rows are grouped by, besides the time bucket. | 
**error** | Option<**String**> | Why the build failed, when status is `build_failed`; null otherwise. | [optional]
**failed_at** | Option<**String**> | When the build failed, when status is `build_failed`; null otherwise. | [optional]
**filter** | Option<**String**> | Condition a row must pass to be kept; absent when every row is kept. | [optional]
**id** | **String** | Rollup ID. | 
**measures** | [**Vec<models::RollupMeasure>**](RollupMeasure.md) | The measures, as declared. | 
**schema_name** | **String** | Schema of the table. | 
**status** | [**models::RollupStatus**](RollupStatus.md) |  | 
**table_name** | **String** | The table the rollup is declared on. | 
**time_column** | **String** | Timestamp column rows are bucketed by. | 
**updated_at** | **String** | When the rollup last changed. | 
**window_secs** | Option<**i64**> | How long, in seconds, after a row's time it can still be changed or deleted. Shared by every rollup on the table. Null when the table's state cannot be read, or the table was dropped and created again since the rollup was declared. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


