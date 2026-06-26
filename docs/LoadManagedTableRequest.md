# LoadManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**format** | Option<**String**> | File format of the upload: `\"csv\"`, `\"json\"`, or `\"parquet\"`. Optional — when omitted, the format is auto-detected from the upload's `Content-Type` and, failing that, from the file contents. Provide it explicitly to override detection or when the contents are ambiguous. `\"json\"` expects newline-delimited JSON (one object per line), not a JSON array. | [optional]
**mode** | **String** | Load mode. Only `\"replace\"` is supported in this release. | 
**upload_id** | **String** | ID of a previously-staged upload (see `POST /v1/files`). The upload is claimed atomically; concurrent loads against the same `upload_id` return 409. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


