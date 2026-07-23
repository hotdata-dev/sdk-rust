# ListDatabasesResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | Option<**i32**> | Number of databases returned in this page. | [optional]
**databases** | [**Vec<models::DatabaseSummary>**](DatabaseSummary.md) |  | 
**has_more** | Option<**bool**> | Whether more databases exist beyond this page. | [optional]
**limit** | Option<**i32**> | Page size applied to this response (after clamping to the maximum). | [optional]
**next_cursor** | Option<**String**> | Opaque cursor for the next page; present only when `has_more` is true. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


