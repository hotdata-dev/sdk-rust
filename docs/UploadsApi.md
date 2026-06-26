# \UploadsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_upload_session_handler**](UploadsApi.md#create_upload_session_handler) | **POST** /v1/uploads | Create upload session
[**create_upload_sessions_batch_handler**](UploadsApi.md#create_upload_sessions_batch_handler) | **POST** /v1/uploads/batch | Create upload sessions in bulk
[**finalize_upload_handler**](UploadsApi.md#finalize_upload_handler) | **POST** /v1/uploads/{upload_id}/finalize | Finalize upload
[**list_uploads**](UploadsApi.md#list_uploads) | **GET** /v1/files | List uploads
[**upload_file**](UploadsApi.md#upload_file) | **POST** /v1/files | Upload file



## create_upload_session_handler

> models::UploadSessionResponse create_upload_session_handler(create_upload_request)
Create upload session

Create an upload session for a file you will send directly to storage. Based on the declared size, the response is one of two shapes. For a small file (`mode: single`) it contains a short-lived `url` to `PUT` the whole file to. For a large file (`mode: multipart`) it contains `part_urls` and `part_size`: split the file into `part_size`-byte chunks (the last is the remainder) and `PUT` chunk *i* (1-based) to `part_urls[i - 1]`, keeping each response's `ETag`. Slice by `part_size`, not by an even division across `part_urls.len()` (which can make a non-final part too small). In both cases the response also includes a one-time `finalize_token`. After uploading, call the finalize endpoint with the token (and, for multipart, the `{part_number, e_tag}` list) to make the upload usable as managed-table contents. The returned upload ID can then be passed to the managed-table load endpoint.  You may hint a preferred part size with `part_size`; the service clamps it to the allowed range and ignores it for single-`PUT` uploads.  If the response status is `501` with error code `PRESIGN_UNSUPPORTED`, the configured storage backend cannot issue upload URLs; send the file to the `POST /v1/files` endpoint instead.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_upload_request** | [**CreateUploadRequest**](CreateUploadRequest.md) |  | [required] |

### Return type

[**models::UploadSessionResponse**](UploadSessionResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_upload_sessions_batch_handler

> models::BatchCreateUploadResponse create_upload_sessions_batch_handler(batch_create_upload_request)
Create upload sessions in bulk

Create upload sessions for several files in one request. Each file is planned independently and the response returns one session per requested file, in the same order. Each session is finalized separately via the finalize endpoint, so you can upload and finalize files at your own pace.  If the response status is `501` with error code `PRESIGN_UNSUPPORTED`, the configured storage backend cannot issue upload URLs; send each file to the `POST /v1/files` endpoint instead.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_create_upload_request** | [**BatchCreateUploadRequest**](BatchCreateUploadRequest.md) |  | [required] |

### Return type

[**models::BatchCreateUploadResponse**](BatchCreateUploadResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## finalize_upload_handler

> models::FinalizeUploadResponse finalize_upload_handler(upload_id, x_upload_finalize_token, finalize_upload_request)
Finalize upload

Confirm that a file has been uploaded to storage and make it usable as managed-table contents. Supply the `finalize_token` returned when the session was created, in the `X-Upload-Finalize-Token` header. The uploaded file's size is validated against the size declared at create time; a mismatch is rejected. Finalize is exactly-once: a second finalize of the same upload is rejected.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**upload_id** | **String** | Upload session ID returned at create time | [required] |
**x_upload_finalize_token** | **String** | One-time finalize token returned when the session was created | [required] |
**finalize_upload_request** | Option<[**FinalizeUploadRequest**](FinalizeUploadRequest.md)> | Optional; send `parts` only for a multi-part upload. Single-`PUT` uploads finalize with no body. |  |

### Return type

[**models::FinalizeUploadResponse**](FinalizeUploadResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_uploads

> models::ListUploadsResponse list_uploads(status)
List uploads

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**status** | Option<**String**> | Filter by upload status |  |

### Return type

[**models::ListUploadsResponse**](ListUploadsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_file

> models::UploadResponse upload_file(body)
Upload file

Upload a Parquet file to publish as the contents of a managed table. Send the raw file bytes as the request body with an appropriate Content-Type header (e.g., `application/parquet`). The body is streamed to disk, so files up to 20GB are supported. The returned upload ID can be passed to the managed-table load endpoint.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::UploadResponse**](UploadResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

