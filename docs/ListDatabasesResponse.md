# ListDatabasesResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | Option<**i32**> | Number of databases returned in this page. | [optional]
**databases** | [**Vec<models::DatabaseSummary>**](DatabaseSummary.md) |  | 
**has_more** | Option<**bool**> | Whether more databases exist after this page. | [optional]
**limit** | Option<**i32**> | Page size used for this response. | [optional]
**next_cursor** | Option<**String**> | Cursor for the next page. Present only when `has_more` is true. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


