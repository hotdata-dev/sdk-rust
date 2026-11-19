# ListIndexesPageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **i32** | Number of indexes on this page. | 
**has_more** | **bool** | Whether more indexes follow. Fetch them with `next_cursor`. | 
**indexes** | [**Vec<models::IndexEntryResponse>**](IndexEntryResponse.md) | Indexes on this page. | 
**limit** | **i32** | Page size used. | 
**next_cursor** | Option<**String**> | Cursor for the next page, present only when `has_more` is true. Pass it as the `cursor` query parameter. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


