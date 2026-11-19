# ForkedFromInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**database_id** | **String** | ID of the source database. It may have been deleted since. | 
**description** | Option<**String**> | Why the fork was taken, if given. | [optional]
**forked_at** | Option<**String**> | When the fork was taken. | [optional]
**name** | Option<**String**> | Name of the source when the fork was taken. | [optional]
**snapshot_id** | Option<**i64**> | Point in time of the source data that the fork copied. Not a revision count: two forks of an unchanged source get different values. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


