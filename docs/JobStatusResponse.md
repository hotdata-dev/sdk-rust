# JobStatusResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attempts** | **i32** | Number of execution attempts (including the current one). | 
**completed_at** | Option<**String**> |  | [optional]
**created_at** | **String** |  | 
**error_message** | Option<**String**> | Error or warning message. Set when status is `failed` or `partially_succeeded`. | [optional]
**id** | **String** |  | 
**job_type** | [**models::JobType**](JobType.md) |  | 
**result** | Option<[**models::JobResult**](JobResult.md)> |  | [optional]
**status** | [**models::JobStatus**](JobStatus.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


