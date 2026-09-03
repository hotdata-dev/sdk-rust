# LineageForkInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**database_id** | **String** |  | 
**exists** | **bool** | False once the fork has been deleted. The record of it is kept either way, so a source can still account for everything taken from it. | 
**forked_at** | Option<**String**> | When the fork was taken. | [optional]
**name** | Option<**String**> | Absent once the fork has been deleted — only the record of it remains. | [optional]
**snapshot_id** | Option<**i64**> | Version of this database's data that the fork copied. See `forked_from.snapshot_id`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


