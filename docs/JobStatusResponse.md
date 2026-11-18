# JobStatusResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attempts** | **i32** | Number of attempts so far, including the current one. | 
**completed_at** | Option<**String**> | When the job finished. Absent until then. | [optional]
**created_at** | **String** | When the job was created. | 
**error_message** | Option<**String**> | What went wrong. Set when status is `failed` or `partially_succeeded`. | [optional]
**id** | **String** | Job ID. | 
**job_type** | [**models::JobType**](JobType.md) |  | 
**result** | Option<[**models::JobResult**](JobResult.md)> |  | [optional]
**status** | [**models::JobStatus**](JobStatus.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


