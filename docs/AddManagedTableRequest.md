# AddManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**key** | Option<**Vec<String>**> | Columns that uniquely identify a row, enabling the key-based load modes (`delete`, `update`, `upsert`) on this table: those loads match rows by these columns' values. Omit (the default) to declare no key; the table can still be loaded with `replace` and `append`, but key-based modes are then rejected. | [optional]
**name** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


