# WorkspaceUsageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bytes_scanned** | **i64** | Sum of `bytes_scanned` across all completed/failed query runs since `since`. Null bytes (queries that touched no row data) contribute 0.  This is the storage read the workspace's queries caused, not how much data they covered: a query whose data was already cached contributes little or nothing (see `bytes_scanned` on a query run). Two periods with identical query histories can therefore report different totals, and this figure will not reconcile against a per-query estimate of rows or bytes touched. | 
**query_count** | **i64** | Number of query runs (succeeded + failed) since `since`. | 
**since** | **String** | The period start used for this response (echoed back for the caller to verify). | 
**storage_bytes** | **i64** | The workspace's current stored-data footprint in bytes, measured at request time: instant-database data, plus un-consumed uploads, connection caches, and search-index artifacts. | 
**storage_captured_at** | Option<**String**> | When `storage_bytes` was measured (the time this response was produced). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


