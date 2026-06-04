# \WorkspacesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_workspace**](WorkspacesApi.md#create_workspace) | **POST** /v1/workspaces | Create a workspace
[**delete_workspace**](WorkspacesApi.md#delete_workspace) | **DELETE** /v1/workspaces/{public_id} | Delete a workspace
[**list_workspaces**](WorkspacesApi.md#list_workspaces) | **GET** /v1/workspaces | List workspaces



## create_workspace

> models::CreateWorkspaceResponse create_workspace(create_workspace_request)
Create a workspace

Creates a new workspace in the specified organization.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_workspace_request** | [**CreateWorkspaceRequest**](CreateWorkspaceRequest.md) |  | [required] |

### Return type

[**models::CreateWorkspaceResponse**](CreateWorkspaceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_workspace

> delete_workspace(public_id)
Delete a workspace

Hard-deletes the workspace. Namespace, storage, and catalog deprovisioning runs asynchronously after the row is removed.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**public_id** | **String** | Public id of the workspace. | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_workspaces

> models::ListWorkspacesResponse list_workspaces(organization_public_id)
List workspaces

Lists all workspaces in the user's organization.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**organization_public_id** | Option<**String**> | Filter by organization. Defaults to the user's current organization. |  |

### Return type

[**models::ListWorkspacesResponse**](ListWorkspacesResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

