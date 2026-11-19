# DeclareRollupRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bucket_width_secs** | **i64** | Width of each time bucket, in seconds. Buckets align to the Unix epoch, in UTC. A query uses the rollup when its own time buckets are a multiple of this width. | 
**dimensions** | Option<**Vec<String>**> | Columns to group by, besides the time bucket. A query can also filter on these columns and still use the rollup. | [optional]
**filter** | Option<**String**> | SQL condition a row must pass to be kept; omit it to keep every row. A query uses the rollup only when its `WHERE` clause contains each `AND`-joined part written the same way, and the condition must be deterministic (no subqueries, aggregates, `now()` or `random()`). | [optional]
**measures** | [**Vec<models::RollupMeasure>**](RollupMeasure.md) | Aggregates each rollup row keeps. | 
**schema_name** | **String** | Schema that holds the table. | 
**table_name** | **String** | Table to roll up. | 
**time_column** | **String** | Timestamp column to bucket rows by. It must have no time zone, or be in UTC. | 
**window_secs** | Option<**i64**> | How long, in seconds, after a row's time it can still be changed or deleted; later edits are refused, appends are always allowed. Shared by every rollup on the table: the first sets it (default one hour), later ones must match or omit it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


