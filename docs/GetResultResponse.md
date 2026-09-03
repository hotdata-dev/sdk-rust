# GetResultResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<**Vec<String>**> |  | [optional]
**error_message** | Option<**String**> |  | [optional]
**nullable** | Option<**Vec<bool>**> |  | [optional]
**result_id** | **String** |  | 
**row_count** | Option<**i64**> |  | [optional]
**rows** | Option<[**Vec<Vec<serde_json::Value>>**](Vec.md)> | Array of rows, where each row is an array of column values. | [optional]
**status** | **String** |  | 
**total_row_count** | Option<**i64**> | Grand total rows in the full result, ignoring `offset` and `limit`. Present whenever the result is `ready`, and carrying the same value as the `X-Total-Row-Count` response header.  Compare it against `row_count` to tell whether this body is the whole result: `row_count < total_row_count` means the rest is still there, one page further on. Without it a windowed fetch cannot tell a full result from a truncated one from the body alone. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


