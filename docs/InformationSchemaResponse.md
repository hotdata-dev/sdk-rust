# InformationSchemaResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **i32** | Number of tables on this page, not the total. An empty `tables` array does not mean the listing is finished: check `has_more`. | 
**has_more** | **bool** | Whether more tables follow. Fetch them with `next_cursor`. | 
**limit** | **i32** | Page size used: the `limit` you sent, capped at the maximum, or the default when omitted. | 
**next_cursor** | Option<**String**> | Cursor for the next page, present only when `has_more` is true. Pass it as the `cursor` query parameter. | [optional]
**tables** | [**Vec<models::TableInfo>**](TableInfo.md) | Tables on this page. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


