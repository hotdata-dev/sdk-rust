# QueryParam

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**default** | Option<**serde_json::Value**> | Value used when an execution does not supply one. Without a default, every execution must supply the parameter. `null` is a valid default. | [optional]
**description** | Option<**String**> | What the parameter is for. | [optional]
**name** | **String** | Name used in the SQL after `$`. Letters, digits and underscores, starting with a letter or underscore. | 
**r#type** | **String** | Type of the value: `string`, `int`, `float`, `bool`, `timestamp`, `date` or `interval`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


