# \JobsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_job**](JobsApi.md#get_job) | **GET** /v1/jobs/{id} | Get job status
[**list_jobs**](JobsApi.md#list_jobs) | **GET** /v1/jobs | List jobs



## get_job

> models::JobStatusResponse get_job(id)
Get job status

Get the current status of a background job. Poll this endpoint to track job progress.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Job ID | [required] |

### Return type

[**models::JobStatusResponse**](JobStatusResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_jobs

> models::ListJobsResponse list_jobs(job_type, status, limit, offset)
List jobs

List background jobs with optional filters by type and status.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_type** | Option<**String**> | Filter by job type |  |
**status** | Option<**String**> | Filter by status |  |
**limit** | Option<**i32**> | Max results (default 50) |  |
**offset** | Option<**i32**> | Offset for pagination |  |

### Return type

[**models::ListJobsResponse**](ListJobsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

