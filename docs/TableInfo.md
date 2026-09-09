# TableInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<[**Vec<models::ColumnInfo>**](ColumnInfo.md)> |  | [optional]
**connection** | **String** |  | 
**constant_per_key** | **Vec<String>** | Columns the table declares constant for a given key: for every row, any other row sharing its key holds the same value of these columns.  Declaring this lets a keyed mutation narrow its search for prior versions to the values the upload carries. Empty when none is declared, which is the unrestricted search.  Unlike `partition_by` and `sorted_by` this is NOT fixed at creation — it changes only which files a mutation opens, never how rows are written — so read it here rather than assuming a declaration took effect. | 
**last_sync** | Option<**String**> |  | [optional]
**partition_by** | [**Vec<models::TablePartitionKey>**](TablePartitionKey.md) | The table's partition keys, in the order they were declared when the table was created. Empty when the table is not partitioned.  A table's storage layout is fixed when the table is created and cannot be changed afterwards, so this is how to confirm a table really was created with the layout that was asked for. The field is always present: an empty array means \"no partitioning declared\", which is not the same as a response that omits the field entirely.  Reported for tables in a Hotdata instant database, which are the only ones whose layout is declared here. A table discovered from an external connection always reports an empty array — its layout belongs to the upstream system, so an empty array there means \"not known from here\", not \"confirmed unpartitioned\". | 
**schema** | **String** |  | 
**sorted_by** | [**Vec<models::TableSortKey>**](TableSortKey.md) | The table's sort keys, in the order they were declared when the table was created. Empty when no sort order was declared. Always present, and limited to tables in a Hotdata instant database, for the same reasons as `partition_by`. | 
**synced** | **bool** |  | 
**table** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


