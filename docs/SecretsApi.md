# \SecretsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_secret**](SecretsApi.md#create_secret) | **POST** /v1/secrets | Create secret
[**delete_secret**](SecretsApi.md#delete_secret) | **DELETE** /v1/secrets/{name} | Delete secret
[**get_secret**](SecretsApi.md#get_secret) | **GET** /v1/secrets/{name} | Get secret
[**list_secrets**](SecretsApi.md#list_secrets) | **GET** /v1/secrets | List secrets
[**update_secret**](SecretsApi.md#update_secret) | **PUT** /v1/secrets/{name} | Update secret



## create_secret

> models::CreateSecretResponse create_secret(create_secret_request)
Create secret

Store a new named secret. The value is encrypted at rest and can be referenced by connections for authentication. Secret names must be unique.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_secret_request** | [**CreateSecretRequest**](CreateSecretRequest.md) |  | [required] |

### Return type

[**models::CreateSecretResponse**](CreateSecretResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_secret

> delete_secret(name)
Delete secret

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Secret name | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_secret

> models::GetSecretResponse get_secret(name)
Get secret

Get metadata for a secret. The secret value is never returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Secret name | [required] |

### Return type

[**models::GetSecretResponse**](GetSecretResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_secrets

> models::ListSecretsResponse list_secrets()
List secrets

List all stored secrets. Only metadata (name, timestamps) is returned — secret values are never exposed.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListSecretsResponse**](ListSecretsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_secret

> models::UpdateSecretResponse update_secret(name, update_secret_request)
Update secret

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Secret name | [required] |
**update_secret_request** | [**UpdateSecretRequest**](UpdateSecretRequest.md) |  | [required] |

### Return type

[**models::UpdateSecretResponse**](UpdateSecretResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

