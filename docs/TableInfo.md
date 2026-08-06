# TableInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<[**Vec<models::ColumnInfo>**](ColumnInfo.md)> |  | [optional]
**connection** | **String** |  | 
**last_sync** | Option<**String**> |  | [optional]
**partition_by** | [**Vec<models::TablePartitionKey>**](TablePartitionKey.md) | The table's partition keys, in the order they were declared when the table was created. Empty when the table is not partitioned.  A table's storage layout is fixed when the table is created and cannot be changed afterwards, so this is how to confirm a table really was created with the layout that was asked for. The field is always present: an empty array means \"no partitioning declared\", which is not the same as a response that omits the field entirely.  Reported for tables in a hotdata-managed database, which are the only ones whose layout is declared here. A table discovered from an external connection always reports an empty array — its layout belongs to the upstream system, so an empty array there means \"not known from here\", not \"confirmed unpartitioned\". | 
**schema** | **String** |  | 
**sorted_by** | [**Vec<models::TableSortKey>**](TableSortKey.md) | The table's sort keys, in the order they were declared when the table was created. Empty when no sort order was declared. Always present, and limited to tables in a hotdata-managed database, for the same reasons as `partition_by`. | 
**synced** | **bool** |  | 
**table** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


