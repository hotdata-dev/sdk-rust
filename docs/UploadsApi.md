# \UploadsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_uploads**](UploadsApi.md#list_uploads) | **GET** /v1/files | List uploads
[**upload_file**](UploadsApi.md#upload_file) | **POST** /v1/files | Upload file



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

