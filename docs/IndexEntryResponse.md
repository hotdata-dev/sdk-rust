# IndexEntryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**algorithm** | Option<**String**> | How this vector index organises the vectors it searches: `hnsw` or `ivf`. Absent for BM25 and sorted indexes. | [optional]
**columns** | **Vec<String>** |  | 
**created_at** | **String** |  | 
**index_name** | **String** |  | 
**index_type** | **String** |  | 
**metric** | Option<**String**> | Distance metric this index was built with. Only present for vector indexes. | [optional]
**probe_fraction** | Option<**f64**> | How much of an `ivf` index a search reads, as a fraction greater than 0 and at most 1, when it was created with an explicit one. Absent means the server's default. Also absent for every other kind of index. | [optional]
**source_column** | Option<**String**> | Source text column for an embedding-backed vector index. A query searches it via `vector_distance(<source_column>, …)`; the indexed `columns` hold the generated embedding column instead. Absent for BM25, sorted, and direct (existing-column) vector indexes. | [optional]
**status** | [**models::IndexStatus**](IndexStatus.md) |  | 
**updated_at** | **String** |  | 
**vector_precision** | Option<**String**> | How precisely this vector index stores each number of a vector, when it was created with an explicit precision. Absent means it stores at the same precision as the column, which is the default. Also absent for BM25 and sorted indexes. | [optional]
**connection_id** | Option<**String**> |  | [optional]
**schema_name** | **String** |  | 
**table_name** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


