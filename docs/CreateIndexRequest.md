# CreateIndexRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | When true, create the index as a background job and return a job ID for polling. | [optional]
**columns** | **Vec<String>** | Columns to index. Required for all index types. | 
**description** | Option<**String**> | User-facing description of the embedding (e.g., \"product descriptions\"). | [optional]
**dimensions** | Option<**i32**> | Output vector dimensions. Some models support multiple dimension sizes (e.g., OpenAI text-embedding-3-small supports 512 or 1536). If omitted, the model's default dimensions are used | [optional]
**embedding_provider_id** | Option<**String**> | Embedding provider ID. When set for a vector index, the source column is treated as text and embeddings are generated automatically. The vector index is then built on the generated embedding column (`{column}_embedding` by default). | [optional]
**index_name** | **String** |  | 
**index_type** | Option<**String**> | Index type: \"sorted\" (default), \"bm25\", or \"vector\" | [optional]
**metric** | Option<**String**> | Distance metric for vector indexes: \"l2\", \"cosine\", or \"dot\". When omitted, defaults to \"l2\" for float array columns or the provider's preferred metric for text columns with auto-embedding. | [optional]
**output_column** | Option<**String**> | Custom name for the generated embedding column. Defaults to `{column}_embedding`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


