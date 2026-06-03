# \SandboxesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sandbox**](SandboxesApi.md#create_sandbox) | **POST** /v1/sandboxes | Create a sandbox
[**delete_sandbox**](SandboxesApi.md#delete_sandbox) | **DELETE** /v1/sandboxes/{public_id} | Delete sandbox
[**get_sandbox**](SandboxesApi.md#get_sandbox) | **GET** /v1/sandboxes/{public_id} | Get sandbox
[**list_sandboxes**](SandboxesApi.md#list_sandboxes) | **GET** /v1/sandboxes | List sandboxes
[**update_sandbox**](SandboxesApi.md#update_sandbox) | **PATCH** /v1/sandboxes/{public_id} | Update sandbox



## create_sandbox

> models::SandboxResponse create_sandbox(create_sandbox_request)
Create a sandbox

Creates a sandbox in the requested workspace. The returned `public_id` is the value to pass as `X-Session-Id` on scoped ops.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_sandbox_request** | [**CreateSandboxRequest**](CreateSandboxRequest.md) |  | [required] |

### Return type

[**models::SandboxResponse**](SandboxResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sandbox

> models::DeleteSandboxResponse delete_sandbox(public_id)
Delete sandbox

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**public_id** | **String** | Public id of the sandbox. | [required] |

### Return type

[**models::DeleteSandboxResponse**](DeleteSandboxResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sandbox

> models::SandboxResponse get_sandbox(public_id)
Get sandbox

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**public_id** | **String** | Public id of the sandbox. | [required] |

### Return type

[**models::SandboxResponse**](SandboxResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sandboxes

> models::ListSandboxesResponse list_sandboxes()
List sandboxes

Lists sandboxes for the caller in the requested workspace.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListSandboxesResponse**](ListSandboxesResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_sandbox

> models::SandboxResponse update_sandbox(public_id, update_sandbox_request)
Update sandbox

Partial update. Only the provided fields are changed.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**public_id** | **String** | Public id of the sandbox. | [required] |
**update_sandbox_request** | [**UpdateSandboxRequest**](UpdateSandboxRequest.md) |  | [required] |

### Return type

[**models::SandboxResponse**](SandboxResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

