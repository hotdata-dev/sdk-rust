# \SavedQueriesApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_saved_query**](SavedQueriesApi.md#create_saved_query) | **POST** /v1/queries | Create saved query
[**delete_saved_query**](SavedQueriesApi.md#delete_saved_query) | **DELETE** /v1/queries/{id} | Delete saved query
[**execute_saved_query**](SavedQueriesApi.md#execute_saved_query) | **POST** /v1/queries/{id}/execute | Execute saved query
[**get_saved_query**](SavedQueriesApi.md#get_saved_query) | **GET** /v1/queries/{id} | Get saved query
[**list_saved_queries**](SavedQueriesApi.md#list_saved_queries) | **GET** /v1/queries | List saved queries
[**list_saved_query_versions**](SavedQueriesApi.md#list_saved_query_versions) | **GET** /v1/queries/{id}/versions | List saved query versions
[**update_saved_query**](SavedQueriesApi.md#update_saved_query) | **PUT** /v1/queries/{id} | Update saved query



## create_saved_query

> models::SavedQueryDetail create_saved_query(create_saved_query_request)
Create saved query

Save a named SQL query. The SQL is stored as version 1 and automatically analyzed for classification metadata (category, table count, predicate/join/aggregation flags).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_saved_query_request** | [**CreateSavedQueryRequest**](CreateSavedQueryRequest.md) |  | [required] |

### Return type

[**models::SavedQueryDetail**](SavedQueryDetail.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_saved_query

> delete_saved_query(id)
Delete saved query

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## execute_saved_query

> models::QueryResponse execute_saved_query(id, x_database_id, execute_saved_query_request)
Execute saved query

Execute a saved query, scoped to a database (required `X-Database-Id` header). By default runs the latest version. Optionally specify a version number to execute a previous version. The SQL runs inside the given database scope, the same way POST /v1/query does. Returns the same response format as POST /v1/query.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |
**x_database_id** | **String** | Required. Scope execution to this database (its id). A missing or malformed value is a 400; an unknown database id is a 404. | [required] |
**execute_saved_query_request** | Option<[**ExecuteSavedQueryRequest**](ExecuteSavedQueryRequest.md)> | Optional version to execute |  |

### Return type

[**models::QueryResponse**](QueryResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_saved_query

> models::SavedQueryDetail get_saved_query(id)
Get saved query

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |

### Return type

[**models::SavedQueryDetail**](SavedQueryDetail.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_saved_queries

> models::ListSavedQueriesResponse list_saved_queries(limit, offset)
List saved queries

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of results |  |
**offset** | Option<**i32**> | Pagination offset |  |

### Return type

[**models::ListSavedQueriesResponse**](ListSavedQueriesResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_saved_query_versions

> models::ListSavedQueryVersionsResponse list_saved_query_versions(id, limit, offset)
List saved query versions

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |
**limit** | Option<**i32**> | Maximum number of versions |  |
**offset** | Option<**i32**> | Pagination offset |  |

### Return type

[**models::ListSavedQueryVersionsResponse**](ListSavedQueryVersionsResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_saved_query

> models::SavedQueryDetail update_saved_query(id, update_saved_query_request)
Update saved query

Update a saved query. If the SQL changes, a new version is created (previous versions are preserved). Name, tags, description, and classification overrides can also be updated.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |
**update_saved_query_request** | [**UpdateSavedQueryRequest**](UpdateSavedQueryRequest.md) |  | [required] |

### Return type

[**models::SavedQueryDetail**](SavedQueryDetail.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

