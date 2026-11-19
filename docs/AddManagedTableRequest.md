# AddManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**constant_per_key** | Option<**Vec<String>**> | Columns that always hold the same value for rows with the same `key`. Makes keyed loads faster, but if it is not true, keyed loads silently duplicate rows. | [optional]
**key** | Option<**Vec<String>**> | Columns that identify a row. Required for `delete`, `update` and `upsert` loads unless the load request gives its own `key`. | [optional]
**layout** | Option<[**models::TableLayout**](TableLayout.md)> |  | [optional]
**name** | **String** |  | 
**partition_by** | Option<[**Vec<models::TablePartitionKey>**](TablePartitionKey.md)> | Columns to partition the table by, in order. Cannot be changed later. | [optional]
**sorted_by** | Option<[**Vec<models::TableSortKey>**](TableSortKey.md)> | Columns to sort the table by, in order. Cannot be changed later, and a sorted table cannot be indexed. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


