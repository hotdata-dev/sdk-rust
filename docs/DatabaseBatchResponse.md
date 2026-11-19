# DatabaseBatchResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**batch_id** | **String** |  | 
**cancel_requested** | **bool** | True once cancellation is requested. Databases already created are kept. | 
**count** | **i64** | Number of databases requested. | 
**created_count** | **i64** | Number of databases created so far. | 
**expires_at** | Option<**String**> |  | [optional]
**job_id** | Option<**String**> | ID of the job creating the databases. Poll it for status. | [optional]
**status_url** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


