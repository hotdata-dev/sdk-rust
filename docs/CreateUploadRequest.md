# CreateUploadRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**checksum_algo** | Option<**String**> | Integrity checksum algorithm you are volunteering for this file. Currently only `sha256` is accepted. Optional; pair with `checksum_value`. | [optional]
**checksum_value** | Option<**String**> | Integrity checksum value, paired with `checksum_algo`. Optional. | [optional]
**content_encoding** | Option<**String**> | Content encoding to record for the uploaded file (for example `gzip`). Optional. | [optional]
**content_type** | Option<**String**> | Content type to record for the uploaded file (for example the Parquet, CSV, or JSON MIME type). Optional. | [optional]
**declared_size_bytes** | Option<**i64**> | The exact size, in bytes, of the file you will upload. Optional. When provided, it is validated at create time against the maximum allowed size, and again at finalize against the bytes actually stored — a mismatch fails the finalize. Omit it to create a streaming (unknown-size) upload: the session is always multi-part and returns no part URLs up front; instead you mint part URLs on demand from `POST /v1/uploads/{upload_id}/parts` as you upload, and finalize validates only that the file is non-empty. | [optional]
**filename** | Option<**String**> | Original file name, recorded with the upload for your own bookkeeping. Optional and advisory — it does not affect where the bytes are stored or how they are loaded. | [optional]
**part_size** | Option<**i64**> | Preferred size, in bytes, of each part for a large (multi-part) upload. Optional hint — the service clamps it to the allowed part-size range and to the maximum number of parts, and ignores it for small files uploaded with a single `PUT`. Omit to let the service choose. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


