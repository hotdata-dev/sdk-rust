# WorkspaceUsageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bytes_scanned** | **i64** | Total `bytes_scanned` of succeeded and failed query runs since `since`. Reads served from cache are not counted. | 
**query_count** | **i64** | Number of succeeded and failed query runs since `since`. | 
**since** | **String** | Start of the period these totals cover. | 
**storage_bytes** | **i64** | Bytes the workspace stores now: table data, uploads not yet loaded, and caches and indexes. | 
**storage_captured_at** | Option<**String**> | When `storage_bytes` was measured. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


