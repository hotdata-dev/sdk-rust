# CreateSavedQueryRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | Option<**String**> |  | [optional]
**name** | **String** |  | 
**params** | Option<[**Vec<models::QueryParam>**](QueryParam.md)> | Parameters the SQL uses, written in it as `$name`. Each value is checked against its type and bound as a value when the query runs, never as SQL text. Every `$name` in the SQL must be declared here, and every declared parameter must appear in the SQL. | [optional]
**sql** | **String** |  | 
**tags** | Option<**Vec<String>**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


