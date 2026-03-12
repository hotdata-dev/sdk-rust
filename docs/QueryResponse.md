# QueryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | **Vec<String>** |  | 
**execution_time_ms** | **i64** |  | 
**nullable** | **Vec<bool>** | Nullable flags for each column (parallel to columns vec). True if the column allows NULL values, false if NOT NULL. | 
**query_run_id** | **String** | Unique identifier for the query run record (qrun...). | 
**result_id** | Option<**String**> | Unique identifier for retrieving this result via GET /results/{id}. Null if catalog registration failed (see `warning` field for details). When non-null, the result is being persisted asynchronously. | [optional]
**row_count** | **i32** |  | 
**rows** | [**Vec<Vec<String>>**](Vec.md) | Array of rows, where each row is an array of column values. Values can be strings, numbers, booleans, or null. | 
**warning** | Option<**String**> | Warning message if result persistence could not be initiated. When present, `result_id` will be null and the result cannot be retrieved later. The query results are still returned in this response. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


