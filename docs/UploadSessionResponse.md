# UploadSessionResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**finalize_token** | **String** | Token to send in the `X-Upload-Finalize-Token` header when you request part URLs or finalize. Returned only once. | 
**headers** | **std::collections::HashMap<String, String>** | Headers to send with each `PUT`. Currently always empty. | 
**mode** | **String** | `single`: `PUT` the whole file to `url`. `multipart`: `PUT` each part to its own URL. | 
**part_size** | Option<**i64**> | For a `multipart` upload, the size of every part except the last, in bytes. Split the file by this size, not evenly by part count, or a part may fall under the 5 MiB minimum and finalize fails. | [optional]
**part_urls** | Option<**Vec<String>**> | For a `multipart` upload of known size, one URL per part, in order: `PUT` part *i* to `part_urls[i - 1]` and keep each response's `ETag` for finalize. Absent for a streaming upload. | [optional]
**upload_id** | **String** | ID of the upload. Use it to finalize the upload and then to load it. | 
**url** | Option<**String**> | URL to `PUT` the file to, for a `single` upload. Expires soon. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


