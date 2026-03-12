# \UploadsApi

All URIs are relative to *https://app.hotdata.dev*

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

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_file

> models::UploadResponse upload_file(request_body)
Upload file

Upload a file to be used as a dataset source. Send the raw file bytes as the request body with an appropriate Content-Type header (e.g., `text/csv`, `application/json`, `application/parquet`). The returned upload ID can be passed to POST /v1/datasets to create a queryable table.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<i32>**](I32.md) |  | [required] |

### Return type

[**models::UploadResponse**](UploadResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

