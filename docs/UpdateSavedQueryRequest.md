# UpdateSavedQueryRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**category_override** | Option<**serde_json::Value**> | Override the auto-detected category. Send `null` to clear (revert to auto). | [optional]
**description** | Option<**serde_json::Value**> |  | [optional]
**name** | Option<**serde_json::Value**> | Optional new name. When omitted the existing name is preserved. | [optional]
**sql** | Option<**serde_json::Value**> | Optional new SQL. When omitted the existing SQL is preserved. | [optional]
**table_size_override** | Option<**serde_json::Value**> | User annotation for table size. Send `null` to clear. | [optional]
**tags** | Option<**Vec<String>**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


