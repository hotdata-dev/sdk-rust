# UpdateSavedQueryRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**category_override** | Option<**String**> | Category to use instead of the detected one, such as `point_lookup` or `join`. Send `null` to go back to the detected category. | [optional]
**description** | Option<**String**> | New description. Omit to keep the current one. | [optional]
**name** | Option<**String**> | New name. Omit to keep the current one. | [optional]
**sql** | Option<**String**> | New SQL. Omit to keep the current SQL. | [optional]
**table_size_override** | Option<**String**> | Your own label for the table size. Send `null` to clear it. | [optional]
**tags** | Option<**Vec<String>**> | New tags, replacing the current ones. Omit to keep them. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


