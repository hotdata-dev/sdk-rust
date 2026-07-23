# ListDatabasesResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **i32** | Number of databases returned in this page. | 
**databases** | [**Vec<models::DatabaseSummary>**](DatabaseSummary.md) |  | 
**has_more** | **bool** | Whether more databases exist beyond this page. | 
**limit** | **i32** | Page size applied to this response (after clamping to the maximum). | 
**next_cursor** | Option<**String**> | Opaque cursor for the next page; present only when `has_more` is true. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


