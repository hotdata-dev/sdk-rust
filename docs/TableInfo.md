# TableInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<[**Vec<models::ColumnInfo>**](ColumnInfo.md)> | Table columns. Present only when the request sets `include_columns=true`. | [optional]
**connection** | **String** | Name of the connection that holds the table. | 
**constant_per_key** | **Vec<String>** | Columns that hold the same value for every row sharing a key, which lets keyed mutations search fewer files; empty when none are declared. Unlike `partition_by` and `sorted_by`, this can change after creation. | 
**last_sync** | Option<**String**> | When the table's data was last loaded or synced; null if never. | [optional]
**layout** | [**models::TableLayout**](TableLayout.md) |  | 
**partition_by** | [**Vec<models::TablePartitionKey>**](TablePartitionKey.md) | Partition keys, in the order declared when the table was created; empty when it is not partitioned. Reported only for tables in a Hotdata-instant database: other tables always return an empty array. | 
**schema** | **String** | Schema name. | 
**sorted_by** | [**Vec<models::TableSortKey>**](TableSortKey.md) | Sort keys, in the order declared when the table was created; empty when none were declared. Reported only for tables in a Hotdata-managed database, like `partition_by`. | 
**synced** | **bool** | Whether the table has data loaded. | 
**table** | **String** | Table name. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


