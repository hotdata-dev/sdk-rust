# DatabaseLineageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ancestors** | [**Vec<models::LineageAncestorInfo>**](LineageAncestorInfo.md) | Databases this one descends from, nearest parent first. Deleted ones stay in the list with `exists: false`. Empty when this is not a fork. | 
**ancestors_truncated** | **bool** | True when `ancestors` stops before reaching `root_id`. | 
**database_id** | **String** | The database this lineage describes. | 
**fork_count** | **i64** | Total number of databases forked directly from this one. | 
**forks** | [**Vec<models::LineageForkInfo>**](LineageForkInfo.md) | Databases forked directly from this one, newest first, including deleted ones. Limited by `forks_limit`. | 
**root_id** | **String** | The original database at the top of the fork chain. Equal to `database_id` when this database is not a fork. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


