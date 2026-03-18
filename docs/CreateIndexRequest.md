# CreateIndexRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**index_name** | **String** |  | 
**index_type** | Option<**String**> | Index type: \"sorted\" (default), \"bm25\", or \"vector\" | [optional]
**metric** | Option<**String**> | Distance metric for vector indexes: \"l2\" (default), \"cosine\", or \"dot\". Only relevant when index_type = \"vector\". | [optional]
**sort_columns** | Option<**Vec<String>**> |  | [optional]
**text_columns** | Option<**Vec<String>**> | Text columns for BM25 indexes | [optional]
**vector_columns** | Option<**Vec<String>**> | Vector column for vector indexes (exactly one entry required) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


