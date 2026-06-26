# UploadSessionResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**finalize_token** | **String** | One-time token that authorizes finalizing this upload. Returned exactly once at create time — store it; it cannot be retrieved again. | 
**headers** | **std::collections::HashMap<String, String>** | Headers you must send verbatim with each `PUT`. Currently always empty; present so a future mode can require signed headers without changing the response shape. | 
**mode** | **String** | Upload mode: `single` (upload the whole file with one `PUT` to `url`) or `multipart` (upload each part with one `PUT` to the matching entry in `part_urls`). Modeled as a string so additional modes can be added later without breaking clients. | 
**part_size** | Option<**i64**> | For a `multipart` upload, the size in bytes to split the file into: send bytes `[(i-1) * part_size, i * part_size)` to `part_urls[i - 1]`, with the last part carrying the remainder. Slice by this value — do **not** divide the file evenly by `part_urls.len()`, which can make a non-final part smaller than the 5 MiB minimum that storage requires (the upload then fails at finalize). Absent for `single` uploads. | [optional]
**part_urls** | Option<**Vec<String>**> | For a `multipart` upload, the per-part URLs in ascending part order: `PUT` your file's part *i* (1-based) to `part_urls[i - 1]` and keep each response's `ETag`, then pass the `{part_number, e_tag}` list to finalize. Absent for `single` uploads. | [optional]
**upload_id** | **String** | Identifier for this upload. Pass it to the finalize endpoint and to the managed-table load endpoint once finalized. | 
**url** | Option<**String**> | The URL to `PUT` the raw file bytes to, for a `single` upload. Short-lived — upload promptly and finalize. Absent for `multipart` uploads (use `part_urls`). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


