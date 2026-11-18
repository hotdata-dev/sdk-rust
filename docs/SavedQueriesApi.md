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

Save a named SQL query as version 1. Hotdata classifies the SQL, for example its category, table count, and whether it filters, joins or aggregates.

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

Delete a saved query and all its versions.

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

Run a saved query in the database named by the `X-Database-Id` header. Runs the latest version unless you pass `version`. Returns the same response as `POST /v1/query`. Write statements are refused.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |
**x_database_id** | **String** | ID of the database to run the query in. Required. | [required] |
**execute_saved_query_request** | Option<[**ExecuteSavedQueryRequest**](ExecuteSavedQueryRequest.md)> | Optional version to run |  |

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

Get a saved query and its latest version.

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

List saved queries, sorted by name.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Saved queries to return. Default 100, maximum 1000. |  |
**offset** | Option<**i32**> | Number of saved queries to skip. Default 0. |  |

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

List the versions of a saved query, newest first.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Saved query ID | [required] |
**limit** | Option<**i32**> | Versions to return. Default 100, maximum 1000. |  |
**offset** | Option<**i32**> | Number of versions to skip. Default 0. |  |

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

Update a saved query. Send at least one field; fields you omit do not change. Changing the SQL creates a new version and keeps the old ones.

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

