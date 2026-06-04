# LoadManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mode** | **String** | Load mode. Only `\"replace\"` is supported in this release. | 
**upload_id** | **String** | ID of a previously-staged upload (see `POST /v1/files`). The upload must reference a parquet file. The upload is claimed atomically; concurrent loads against the same `upload_id` return 409. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


