# \ConnectionsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_connection_health_handler**](ConnectionsApi.md#check_connection_health_handler) | **GET** /v1/connections/{connection_id}/health | Check connection health
[**create_connection_handler**](ConnectionsApi.md#create_connection_handler) | **POST** /v1/connections | Create connection
[**delete_connection_handler**](ConnectionsApi.md#delete_connection_handler) | **DELETE** /v1/connections/{connection_id} | Delete connection
[**get_connection_handler**](ConnectionsApi.md#get_connection_handler) | **GET** /v1/connections/{connection_id} | Get connection
[**get_table_profile_handler**](ConnectionsApi.md#get_table_profile_handler) | **GET** /v1/connections/{connection_id}/tables/{schema}/{table}/profile | Get table profile
[**list_connections_handler**](ConnectionsApi.md#list_connections_handler) | **GET** /v1/connections | List connections
[**purge_connection_cache_handler**](ConnectionsApi.md#purge_connection_cache_handler) | **DELETE** /v1/connections/{connection_id}/cache | Purge connection cache
[**purge_table_cache_handler**](ConnectionsApi.md#purge_table_cache_handler) | **DELETE** /v1/connections/{connection_id}/tables/{schema}/{table}/cache | Purge table cache



## check_connection_health_handler

> models::ConnectionHealthResponse check_connection_health_handler(connection_id)
Check connection health

Test connectivity to the remote database. Returns health status and latency.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |

### Return type

[**models::ConnectionHealthResponse**](ConnectionHealthResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_connection_handler

> models::CreateConnectionResponse create_connection_handler(create_connection_request)
Create connection

Register a new database connection. Provide the source type and connection config (host, port, database, etc.). Credentials can be supplied inline (password/token fields are auto-converted to secrets) or by referencing an existing secret by name or ID. Schema discovery runs automatically after registration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_connection_request** | [**CreateConnectionRequest**](CreateConnectionRequest.md) |  | [required] |

### Return type

[**models::CreateConnectionResponse**](CreateConnectionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_connection_handler

> delete_connection_handler(connection_id)
Delete connection

Delete a connection and its cached data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_connection_handler

> models::GetConnectionResponse get_connection_handler(connection_id)
Get connection

Get details for a specific connection, including table and sync counts.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |

### Return type

[**models::GetConnectionResponse**](GetConnectionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_table_profile_handler

> models::TableProfileResponse get_table_profile_handler(connection_id, schema, table)
Get table profile

Get column-level statistics for a synced table. Returns per-column profiles including cardinality, null counts, and type-specific details (distinct values for categorical columns, min/max for temporal/numeric, length stats for text). Profiles are computed at sync time.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |

### Return type

[**models::TableProfileResponse**](TableProfileResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_connections_handler

> models::ListConnectionsResponse list_connections_handler()
List connections

List all registered database connections.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListConnectionsResponse**](ListConnectionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## purge_connection_cache_handler

> purge_connection_cache_handler(connection_id)
Purge connection cache

Purge all cached data for a connection. The next query against these tables will trigger a fresh sync from the remote source.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## purge_table_cache_handler

> purge_table_cache_handler(connection_id, schema, table)
Purge table cache

Purge the cached data for a single table. The next query will trigger a fresh sync.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | **String** | Connection ID | [required] |
**schema** | **String** | Schema name | [required] |
**table** | **String** | Table name | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

