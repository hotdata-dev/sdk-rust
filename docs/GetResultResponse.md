# GetResultResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<**Vec<String>**> | Column names, in order. Present when `status` is `ready`. | [optional]
**error_message** | Option<**String**> | Why the result failed. Present only when `status` is `failed`. | [optional]
**nullable** | Option<**Vec<bool>**> | Whether each column, in the same order as `columns`, can hold nulls. | [optional]
**result_id** | **String** |  | 
**row_count** | Option<**i64**> | Number of rows in this response. | [optional]
**rows** | Option<[**Vec<Vec<models::JsonCell>>**](Vec.md)> | Result rows. Each row is an array of values in column order. | [optional]
**status** | **String** | `pending`, `processing`, `ready` or `failed`. | 
**total_row_count** | Option<**i64**> | Number of rows in the full result, ignoring `offset` and `limit`. If `row_count` is smaller, more rows are available on later pages. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


