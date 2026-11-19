# CreateIndexRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**algorithm** | Option<**Algorithm**> | How a vector index is built. Defaults to `hnsw`.  `hnsw`: a graph kept in memory. Very fast searches, but memory grows with the table, so very large tables cannot be served.  `ivf`: vectors grouped into clusters; a search reads only the nearest clusters. Slower than `hnsw`, but memory follows how much of the index a search reads, so it works for much larger tables. It stores a copy of the table's rows, which uses extra storage. Supports the `l2` and `cosine` metrics. Search it with `ORDER BY vector_distance(column, '<query>') LIMIT <k>` and name the columns to return instead of `*`; `vector_search` works only with `hnsw`. Generating embeddings for tens of millions of rows is not supported yet. (enum: hnsw, ivf) | [optional]
**r#async** | Option<**bool**> | Build the index in the background and return a job ID to poll. | [optional][default to false]
**async_after_ms** | Option<**i32**> | With `async: true`, wait up to this many milliseconds for the build before falling back to a job ID. Must be between 1000 and 30000. | [optional]
**columns** | **Vec<String>** | Columns to index. | 
**description** | Option<**String**> | Description of what the embeddings represent, such as \"product descriptions\". | [optional]
**dimensions** | Option<**i32**> | Vector size to request from the embedding model, for models that support more than one. Defaults to the model's own size. | [optional]
**embedding_provider_id** | Option<**String**> | Embedding provider that turns the text column into vectors. The vectors are stored in a new column, and later loads and searches use the same provider. With `ivf`, the metric must be `l2` or `cosine`. | [optional]
**index_name** | **String** |  | 
**index_type** | Option<**IndexType**> | Index type. `sorted` supports range queries, `bm25` full-text search, and `vector` similarity search. (enum: sorted, bm25, vector) | [optional][default to Sorted]
**metric** | Option<**String**> | Distance metric for a vector index: `l2`, `cosine` or `dot`. Defaults to `l2`, or to the embedding provider's metric when `embedding_provider_id` is set. | [optional]
**nlist** | Option<**i32**> | For `ivf`: number of clusters to group the vectors into. More clusters means each search reads less data. Defaults to a value based on the table size. | [optional]
**output_column** | Option<**String**> | Name of the generated vector column. Defaults to `{column}_embedding`. | [optional]
**probe_fraction** | Option<**f64**> | For `ivf`: share of the index each search reads, above 0 and up to 1. Higher is more accurate and slower. | [optional]
**vector_precision** | Option<**VectorPrecision**> | How precisely a vector index stores each number. Lower precision makes the index smaller, so it needs less memory, and for `ivf` makes each search read less. Defaults to the column's own precision for `hnsw`, and `int8` for `ivf`. To change it, drop and recreate the index; the table's data is never changed.  `ivf` accepts `int8` and `float32` only. `hnsw` accepts every value except `int8`.  Accuracy figures below come from benchmarks; test on your own data.  `float64`: only for columns that already hold 64-bit values.  `float32`: half the size of `float64`, with no loss for the usual 32-bit embeddings.  `float16`: half the size of `float32`; results within 0.1 percentage points of `float32` on 1536-dimension text embeddings with cosine distance.  `float8`: a quarter of the size of `float32`; about 4 percentage points less accurate on the same test, which a wider search does not recover.  `int8`: a quarter of the size of `float32`. A search of every cluster finds about 99.5% of the true neighbours; the default search finds fewer (see `probe_fraction`). With `l2`, accuracy drops when vector lengths vary widely (about 93% when the longest is 16 times the shortest); use `float32` if that matters. (enum: float64, float32, float16, float8, int8) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


