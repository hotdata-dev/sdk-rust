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
**nlist** | Option<**i64**> | Number of clusters an `ivf` index was built with. This can be smaller than the `nlist` requested when the index was created, because the number is capped by how many vectors the clusters were fitted to. Absent for every other kind of index. | [optional]
**probe_fraction** | Option<**f64**> | How much of an `ivf` index a search reads, as a fraction greater than 0 and at most 1, when the index was created with one. When absent, the server chooses how much each search reads: a width measured on this index's own data when it was built, scaled to the number of results a search asks for, or a server default when no measurement could be made. Absent for every other kind of index. | [optional]
**source_column** | Option<**String**> | Source text column for an embedding-backed vector index. A query searches it via `vector_distance(<source_column>, …)`; the indexed `columns` hold the generated embedding column instead. Absent for BM25, sorted, and direct (existing-column) vector indexes. | [optional]
**status** | [**models::IndexStatus**](IndexStatus.md) |  | 
**updated_at** | **String** |  | 
**vector_precision** | Option<**String**> | How precisely this vector index stores each number of a vector. Always present for an `ivf` index, which stores `int8` unless it was created with another precision. For an `hnsw` index it is present only when the index was created with an explicit precision; absent means it stores at the same precision as the column. Absent for BM25 and sorted indexes. | [optional]
**connection_id** | Option<**String**> |  | [optional]
**schema_name** | **String** |  | 
**table_name** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


