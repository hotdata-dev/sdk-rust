# AsyncQueryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**query_run_id** | **String** | ID of the query run. | 
**reason** | Option<**String**> | Why the query went to the background, when known. | [optional]
**status** | **String** | Status of the query run. Always `running` here. | 
**status_url** | **String** | URL to poll for the run's status. Send the same `X-Database-Id` header. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


