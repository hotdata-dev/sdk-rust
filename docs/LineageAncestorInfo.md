# LineageAncestorInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**database_id** | **String** |  | 
**description** | Option<**String**> | Why that fork was taken, if given. | [optional]
**exists** | **bool** | False if the database has been deleted. | 
**forked_at** | Option<**String**> | When the next database down the chain was forked from this one. | [optional]
**name** | Option<**String**> | The database's current name, or its name at fork time if deleted. | [optional]
**snapshot_id** | Option<**i64**> | Point in time of this database's data that the next database down the chain copied. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


