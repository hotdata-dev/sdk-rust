# IndexEntryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**algorithm** | Option<**String**> | Vector index algorithm: `hnsw` or `ivf`. Absent for other index types. | [optional]
**columns** | **Vec<String>** | Indexed columns. | 
**created_at** | **String** | When the index was created. | 
**index_name** | **String** | Index name. | 
**index_type** | **String** | Index type: `sorted`, `bm25` or `vector`. | 
**metric** | Option<**String**> | Distance metric of a vector index. Absent for other index types. | [optional]
**nlist** | Option<**i64**> | Number of clusters an `ivf` index was built with. Can be lower than the requested `nlist`, because it is capped by the number of vectors the clusters were fitted to. | [optional]
**probe_fraction** | Option<**f64**> | Share of an `ivf` index each search reads, from 0 (exclusive) to 1, when set at creation. When absent, Hotdata picks the share for each search. | [optional]
**source_column** | Option<**String**> | Text column an embedding-based vector index was built from; search it with `vector_distance(<source_column>, …)`. `columns` then holds the generated embedding column. Absent for other indexes. | [optional]
**status** | [**models::IndexStatus**](IndexStatus.md) |  | 
**updated_at** | **String** | When the index was last updated. | 
**vector_precision** | Option<**String**> | Precision a vector index stores each number at, such as `int8`. Always set for `ivf` (default `int8`); for `hnsw`, set only when chosen at creation, and otherwise it matches the column. | [optional]
**connection_id** | Option<**String**> | ID of the connection that holds the table. | [optional]
**schema_name** | **String** | Schema of the table. | 
**table_name** | **String** | Table the index is on. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


