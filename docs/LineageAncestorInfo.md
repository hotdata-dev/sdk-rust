# LineageAncestorInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**database_id** | **String** |  | 
**exists** | **bool** | False once the ancestor has been deleted. Its place in the chain is kept either way, and the ancestry continues past it. | 
**forked_at** | Option<**String**> | When the next database down the chain was forked from it. | [optional]
**name** | Option<**String**> | The ancestor's current label, or the one captured at fork time when it no longer exists. | [optional]
**snapshot_id** | Option<**i64**> | Version of this ancestor's data that the next database down the chain copied. See `forked_from.snapshot_id`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


