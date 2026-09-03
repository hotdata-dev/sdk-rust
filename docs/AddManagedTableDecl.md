# AddManagedTableDecl

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**key** | Option<**Vec<String>**> | Columns that uniquely identify a row, enabling the key-based load modes (`delete`, `update`, `upsert`) on this table: those loads match rows by these columns' values. Omit (the default) to declare no key; the table can still be loaded with `replace` and `append`, but key-based modes are then rejected. | [optional]
**key_determines** | Option<**Vec<String>**> | Columns whose value is determined by this table's `key`: for every uploaded row, every stored row sharing its key holds the same value of these columns.  Declaring this lets a keyed mutation (`delete`, `update`, `upsert`) restrict its search for prior versions to the values the upload carries, which prunes far harder than the key alone when the key's own file statistics are weak. Omit (the default) for the unrestricted search.  **Correctness-affecting, not a hint.** If the assertion is false, a mutation supersedes one version of a key and appends beside another, silently duplicating it. Declare it only where the invariant is established. | [optional]
**name** | **String** |  | 
**partition_by** | Option<[**Vec<models::TablePartitionKey>**](TablePartitionKey.md)> | Partition keys for this table, applied in order. Omit for no partitioning. Declared when the table is created and fixed thereafter. | [optional]
**sorted_by** | Option<[**Vec<models::TableSortKey>**](TableSortKey.md)> | Sort keys for this table, applied in order. Omit for no sort order. Declared when the table is created and fixed thereafter. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


