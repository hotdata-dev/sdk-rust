# InformationSchemaResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **i32** | Number of tables in this response, the same meaning `count` carries on the results and databases listings.  This is a page size, not a total for the whole filter. Page with `has_more` and `next_cursor`: an empty `tables` array on its own does not mean the listing is finished. | 
**has_more** | **bool** | True when more tables follow this page. Pass `next_cursor` to fetch them. | 
**limit** | **i32** | The page size in effect for this response — the `limit` you asked for, clamped to the server's maximum, or the server default when you sent none. | 
**next_cursor** | Option<**String**> | Cursor for the next page, present only when `has_more` is `true`. Send it back as the `cursor` query parameter. | [optional]
**tables** | [**Vec<models::TableInfo>**](TableInfo.md) | The tables on this page. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


