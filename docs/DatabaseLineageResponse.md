# DatabaseLineageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ancestors** | [**Vec<models::LineageAncestorInfo>**](LineageAncestorInfo.md) | Fork ancestry, nearest parent first, ending at the database named by `root_id` unless `ancestors_truncated` says otherwise. A deleted generation does not cut the chain short — it is listed with `exists` false and the ancestry continues past it. Empty when this database is not a fork. | 
**ancestors_truncated** | **bool** | True when the ancestry is longer than one response carries, so `ancestors` stops before reaching `root_id`. | 
**database_id** | **String** | The database this lineage describes. | 
**fork_count** | **i64** | How many databases were forked directly from this one in total — so you can tell whether `forks` is the whole set or only the newest of it. | 
**forks** | [**Vec<models::LineageForkInfo>**](LineageForkInfo.md) | The databases forked directly from this one, most recently forked first, including any that have since been deleted. A fork of a fork appears in its own parent's lineage, not here. | 
**root_id** | **String** | Top of the family tree: the database the whole chain of forks descends from. Equal to `database_id` when this database is not a fork. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


