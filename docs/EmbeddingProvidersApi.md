# \EmbeddingProvidersApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_embedding_provider**](EmbeddingProvidersApi.md#create_embedding_provider) | **POST** /v1/embedding-providers | Create embedding provider
[**delete_embedding_provider**](EmbeddingProvidersApi.md#delete_embedding_provider) | **DELETE** /v1/embedding-providers/{id} | Delete embedding provider
[**get_embedding_provider**](EmbeddingProvidersApi.md#get_embedding_provider) | **GET** /v1/embedding-providers/{id} | Get embedding provider
[**list_embedding_providers**](EmbeddingProvidersApi.md#list_embedding_providers) | **GET** /v1/embedding-providers | List embedding providers
[**update_embedding_provider**](EmbeddingProvidersApi.md#update_embedding_provider) | **PUT** /v1/embedding-providers/{id} | Update embedding provider



## create_embedding_provider

> models::CreateEmbeddingProviderResponse create_embedding_provider(create_embedding_provider_request)
Create embedding provider

Register a new embedding provider that can be used to generate vector embeddings for text columns. Providers can be service-based (e.g., OpenAI) or local.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_embedding_provider_request** | [**CreateEmbeddingProviderRequest**](CreateEmbeddingProviderRequest.md) |  | [required] |

### Return type

[**models::CreateEmbeddingProviderResponse**](CreateEmbeddingProviderResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_embedding_provider

> delete_embedding_provider(id)
Delete embedding provider

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Embedding provider ID | [required] |

### Return type

 (empty response body)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_embedding_provider

> models::EmbeddingProviderResponse get_embedding_provider(id)
Get embedding provider

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Embedding provider ID | [required] |

### Return type

[**models::EmbeddingProviderResponse**](EmbeddingProviderResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_embedding_providers

> models::ListEmbeddingProvidersResponse list_embedding_providers()
List embedding providers

List all registered embedding providers.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListEmbeddingProvidersResponse**](ListEmbeddingProvidersResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_embedding_provider

> models::UpdateEmbeddingProviderResponse update_embedding_provider(id, update_embedding_provider_request)
Update embedding provider

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Embedding provider ID | [required] |
**update_embedding_provider_request** | [**UpdateEmbeddingProviderRequest**](UpdateEmbeddingProviderRequest.md) |  | [required] |

### Return type

[**models::UpdateEmbeddingProviderResponse**](UpdateEmbeddingProviderResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

