# \UploadsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_upload_session_handler**](UploadsApi.md#create_upload_session_handler) | **POST** /v1/uploads | Create upload session
[**create_upload_sessions_batch_handler**](UploadsApi.md#create_upload_sessions_batch_handler) | **POST** /v1/uploads/batch | Create upload sessions in bulk
[**finalize_upload_handler**](UploadsApi.md#finalize_upload_handler) | **POST** /v1/uploads/{upload_id}/finalize | Finalize upload
[**mint_upload_parts_handler**](UploadsApi.md#mint_upload_parts_handler) | **POST** /v1/uploads/{upload_id}/parts | Mint upload part URLs



## create_upload_session_handler

> models::UploadSessionResponse create_upload_session_handler(create_upload_request)
Create upload session

Start an upload. You `PUT` the file straight to the URLs in the response, not through this API. The response has a one-time `finalize_token` and one of three shapes:  - `mode: single`: `PUT` the whole file to `url`. - `mode: multipart` with `part_urls`: split the file into `part_size`-byte parts (the last one is smaller) and `PUT` part *i* to `part_urls[i - 1]`. Keep each part's `ETag`. - `mode: multipart` with no `part_urls`, when you omit `declared_size_bytes`: get part URLs as you go from `POST /v1/uploads/{upload_id}/parts`.  Then call `POST /v1/uploads/{upload_id}/finalize`, and load the upload into a table by its ID.  An upload can be at most 16 GiB by default. With `declared_size_bytes`, a larger file is refused here; without it, at finalize. To load more into one table, split the data and load each upload with `mode: append`. A 501 `PRESIGN_UNSUPPORTED` means upload URLs are not available; send the data inline on the load endpoint.

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

Create upload sessions for up to 100 files in one request. Sessions come back in the order of the request. Upload and finalize each one on its own.  The 16 GiB size limit applies to each file. If any file is refused, no sessions are created; retry the whole request. A 501 `PRESIGN_UNSUPPORTED` means upload URLs are not available.

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

Confirm that a file is uploaded, so it can be loaded into a table. Send the `finalize_token` in the `X-Upload-Finalize-Token` header. For a multipart upload, send the `{part_number, e_tag}` list.  If you declared a size, the file must match it. An upload without a declared size must not be empty and must be at most 16 GiB by default. You can finalize an upload only once. Load it within 7 days (by default), or it is deleted.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**upload_id** | **String** | Upload ID | [required] |
**x_upload_finalize_token** | **String** | The `finalize_token` from the create response | [required] |
**finalize_upload_request** | Option<[**FinalizeUploadRequest**](FinalizeUploadRequest.md)> | Send `parts` for a multipart upload. A single-`PUT` upload needs no body. |  |

### Return type

[**models::FinalizeUploadResponse**](FinalizeUploadResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mint_upload_parts_handler

> models::MintUploadPartsResponse mint_upload_parts_handler(upload_id, x_upload_finalize_token, mint_upload_parts_request)
Mint upload part URLs

Get upload URLs for parts of a multipart upload. You need this for an upload created without `declared_size_bytes`. You can also use it to get a new URL for a part whose URL expired.  Send the `finalize_token` in the `X-Upload-Finalize-Token` header and the part numbers (starting at 1), up to 100 per request. `PUT` each part to its URL, keep its `ETag`, and pass the list to finalize.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**upload_id** | **String** | Upload ID | [required] |
**x_upload_finalize_token** | **String** | The `finalize_token` from the create response | [required] |
**mint_upload_parts_request** | [**MintUploadPartsRequest**](MintUploadPartsRequest.md) |  | [required] |

### Return type

[**models::MintUploadPartsResponse**](MintUploadPartsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

