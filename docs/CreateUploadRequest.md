# CreateUploadRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**checksum_algo** | Option<**String**> | Checksum algorithm for `checksum_value`. Only `sha256` is accepted. | [optional]
**checksum_value** | Option<**String**> | Checksum of the file. Stored with the upload but not verified. | [optional]
**content_encoding** | Option<**String**> | Content encoding, such as `gzip`, kept with the upload. | [optional]
**content_type** | Option<**String**> | MIME type of the file. A load uses it to detect the file format when `format` is not set. | [optional]
**declared_size_bytes** | Option<**i64**> | Exact file size in bytes, checked again when you finalize. Omit it to stream a file of unknown size and get part URLs from `POST /v1/uploads/{upload_id}/parts` as you go. | [optional]
**filename** | Option<**String**> | File name, kept for your reference. It does not affect the upload or load. | [optional]
**part_size** | Option<**i64**> | Preferred part size in bytes for a multi-part upload. Hotdata adjusts it to the allowed range; the size it uses is returned as `part_size`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


