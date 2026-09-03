# ForkedFromInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**database_id** | **String** | ID of the database that was forked. The database may since have been deleted — the record outlives it — so this is not guaranteed to resolve. | 
**forked_at** | Option<**String**> | When the fork was taken. | [optional]
**name** | Option<**String**> | Display label the source carried when the fork was taken, kept so a deleted source still reads as more than an ID. | [optional]
**snapshot_id** | Option<**i64**> | Marks the version of the source that this fork copied — its table set and their contents as of that moment. It is a point in time rather than a per-database revision count, so two forks of a source that did not change still report different values, and the numbers are not a way to tell whether a source has changed. Absent only on forks taken before the version was recorded. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


