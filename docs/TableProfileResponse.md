# TableProfileResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | [**Vec<models::ColumnProfileInfo>**](ColumnProfileInfo.md) | Per-column profile statistics | 
**connection** | **String** | Connection name | 
**row_count** | **i32** | Total number of rows in the table | 
**schema** | **String** | Schema name | 
**synced_at** | Option<**serde_json::Value**> | When the table was last synced | [optional]
**table** | **String** | Table name | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


