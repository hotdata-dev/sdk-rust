# \JobsApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_job**](JobsApi.md#get_job) | **GET** /v1/jobs/{id} | Get job status
[**list_jobs**](JobsApi.md#list_jobs) | **GET** /v1/jobs | List jobs



## get_job

> models::JobStatusResponse get_job(id)
Get job status

Get the status of a background job. Poll this until the job finishes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Job ID | [required] |

### Return type

[**models::JobStatusResponse**](JobStatusResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_jobs

> models::ListJobsResponse list_jobs(job_type, status, limit, offset)
List jobs

List background jobs you started, newest first. Filter by `job_type` and `status`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_type** | Option<[**JobType**](JobType.md)> | Filter by one job type. An unknown value returns 400. |  |
**status** | Option<**String**> | Filter by status, comma-separated, for example `status=pending,running`. Values: `pending`, `running`, `succeeded`, `partially_succeeded`, `failed`. An unknown value returns 400. |  |
**limit** | Option<**i32**> | Maximum jobs to return. Default 50, maximum 1000. |  |
**offset** | Option<**i32**> | Number of jobs to skip |  |

### Return type

[**models::ListJobsResponse**](ListJobsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

