# \DatabaseContextApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_database_context**](DatabaseContextApi.md#delete_database_context) | **DELETE** /v1/databases/{database_id}/context/{name} | Delete database context
[**get_database_context**](DatabaseContextApi.md#get_database_context) | **GET** /v1/databases/{database_id}/context/{name} | Get one database context
[**list_database_contexts**](DatabaseContextApi.md#list_database_contexts) | **GET** /v1/databases/{database_id}/context | List database contexts
[**upsert_database_context**](DatabaseContextApi.md#upsert_database_context) | **POST** /v1/databases/{database_id}/context | Create or update database context



## delete_database_context

> delete_database_context(database_id, name)
Delete database context

Removes a named context document from a database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**name** | **String** | Context key: same character rules as a table name | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_context

> models::GetDatabaseContextResponse get_database_context(database_id, name)
Get one database context

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**name** | **String** | Context key: same character rules as a table name | [required] |

### Return type

[**models::GetDatabaseContextResponse**](GetDatabaseContextResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_database_contexts

> models::ListDatabaseContextsResponse list_database_contexts(database_id)
List database contexts

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |

### Return type

[**models::ListDatabaseContextsResponse**](ListDatabaseContextsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upsert_database_context

> models::UpsertDatabaseContextResponse upsert_database_context(database_id, upsert_database_context_request)
Create or update database context

Stores a named document (for example Markdown) scoped to a database. Reuses the same name to replace content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **String** | Database ID | [required] |
**upsert_database_context_request** | [**UpsertDatabaseContextRequest**](UpsertDatabaseContextRequest.md) |  | [required] |

### Return type

[**models::UpsertDatabaseContextResponse**](UpsertDatabaseContextResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

