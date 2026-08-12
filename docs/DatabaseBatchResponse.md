# DatabaseBatchResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**batch_id** | **String** |  | 
**cancel_requested** | **bool** | True once stopping has been requested. Databases already created are kept; only further creation stops. | 
**count** | **i64** | How many databases the batch was asked to create. | 
**created_count** | **i64** | How many exist so far. Advances as the batch fills. | 
**expires_at** | Option<**String**> |  | [optional]
**job_id** | Option<**String**> | Job filling this batch. Poll it for status. | [optional]
**status_url** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


