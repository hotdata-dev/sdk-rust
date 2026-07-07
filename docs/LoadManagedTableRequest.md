# LoadManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | When true, run the load as a background job and return a job ID to poll instead of blocking until it finishes. Recommended for large uploads, which can take longer than an HTTP request should stay open. | [optional]
**async_after_ms** | Option<**i32**> | If set (requires `async` = true), wait up to this many milliseconds for the load to finish: if it completes in time the full result is returned (200), otherwise a 202 with a job ID to poll. Must be between 1000 and the server maximum; a value out of that range, or set without `async` = true, is rejected with 400. | [optional]
**format** | Option<**String**> | File format of the upload: `\"csv\"`, `\"json\"`, or `\"parquet\"`. Optional — when omitted, the format is auto-detected from the upload's `Content-Type` and, failing that, from the file contents. Provide it explicitly to override detection or when the contents are ambiguous. `\"json\"` expects newline-delimited JSON (one object per line), not a JSON array. | [optional]
**mode** | **String** | How the upload is applied: `\"replace\"` overwrites the table's contents, `\"append\"` inserts the uploaded rows on top of the existing data. | 
**upload_id** | **String** | ID of a previously-staged upload (see `POST /v1/files`). The upload is claimed atomically; concurrent loads against the same `upload_id` return 409. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


