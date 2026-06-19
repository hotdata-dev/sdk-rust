# IndexEntryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | **Vec<String>** |  | 
**created_at** | **String** |  | 
**index_name** | **String** |  | 
**index_type** | **String** |  | 
**metric** | Option<**String**> | Distance metric this index was built with. Only present for vector indexes. | [optional]
**source_column** | Option<**String**> | Source text column for an embedding-backed vector index. A query searches it via `vector_distance(<source_column>, …)`; the indexed `columns` hold the generated embedding column instead. Absent for BM25, sorted, and direct (existing-column) vector indexes. | [optional]
**status** | [**models::IndexStatus**](IndexStatus.md) |  | 
**updated_at** | **String** |  | 
**connection_id** | Option<**String**> |  | [optional]
**schema_name** | **String** |  | 
**table_name** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


