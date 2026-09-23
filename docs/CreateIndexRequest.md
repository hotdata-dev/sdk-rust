# CreateIndexRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | When true, create the index as a background job and return a job ID for polling. | [optional][default to false]
**async_after_ms** | Option<**i32**> | If set (requires `async` = true), wait up to this many milliseconds for the index build to finish: if it completes in time the index is returned (201), otherwise a 202 with a job ID to poll. Must be between 1000 and the server maximum; a value out of that range, or set without `async` = true, is rejected with 400. | [optional]
**columns** | **Vec<String>** | Columns to index. Required for all index types. | 
**description** | Option<**String**> | User-facing description of the embedding (e.g., \"product descriptions\"). | [optional]
**dimensions** | Option<**i32**> | Output vector dimensions. Some models support multiple dimension sizes (e.g., OpenAI text-embedding-3-small supports 512 or 1536). If omitted, the model's default dimensions are used | [optional]
**embedding_provider_id** | Option<**String**> | Embedding provider ID. When set for a vector index, the source column is treated as text and embeddings are generated automatically. The vector index is then built on the generated embedding column (`{column}_embedding` by default). | [optional]
**index_name** | **String** |  | 
**index_type** | Option<**IndexType**> | Index type. `sorted` supports range queries, `bm25` full-text search, and `vector` similarity search. (enum: sorted, bm25, vector) | [optional][default to Sorted]
**metric** | Option<**String**> | Distance metric for vector indexes: \"l2\", \"cosine\", or \"dot\". When omitted, defaults to \"l2\" for float array columns or the provider's preferred metric for text columns with auto-embedding. | [optional]
**output_column** | Option<**String**> | Custom name for the generated embedding column. Defaults to `{column}_embedding`. | [optional]
**vector_precision** | Option<**VectorPrecision**> | How precisely a vector index stores each number of a vector. Lower precision shrinks the index so a larger table can be indexed within the same memory, and lets searches run on a smaller instance. Omit this field to store vectors at the same precision as the column, which is the default.  The quality figures below come from one benchmark — 1536-dimension text embeddings, cosine distance, default search settings — and are a guide, not a guarantee. Other models, dimensions, distance metrics and data distributions behave differently, so measure on your own data before moving a production index to a lower precision.  `float32` — on a `float64` column this halves the index. Widely used embedding models emit 32-bit values, so for those nothing is lost; vectors that genuinely carry more than 32 bits of precision will lose some.  `float16` — half the memory of `float32`. In that benchmark its results matched `float32` to within 0.1 percentage points.  `float8` — a quarter of the memory of `float32`. In that benchmark it scored about 4 percentage points below `float32`, and raising the search effort did not close the gap, so treat the reduction as permanent for a given index.  `float64` — accepted only for a column that already holds double-precision values; it cannot add precision the stored data does not have.  Changing this means dropping the index and creating it again. It affects only the index: the table's own values are never altered, and text columns indexed with a generated embedding are not re-embedded. (enum: float64, float32, float16, float8) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


