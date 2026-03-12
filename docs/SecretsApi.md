# \SecretsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_secret_handler**](SecretsApi.md#create_secret_handler) | **POST** /v1/secrets | Create secret
[**delete_secret_handler**](SecretsApi.md#delete_secret_handler) | **DELETE** /v1/secrets/{name} | Delete secret
[**get_secret_handler**](SecretsApi.md#get_secret_handler) | **GET** /v1/secrets/{name} | Get secret
[**list_secrets_handler**](SecretsApi.md#list_secrets_handler) | **GET** /v1/secrets | List secrets
[**update_secret_handler**](SecretsApi.md#update_secret_handler) | **PUT** /v1/secrets/{name} | Update secret



## create_secret_handler

> models::CreateSecretResponse create_secret_handler(create_secret_request)
Create secret

Store a new named secret. The value is encrypted at rest and can be referenced by connections for authentication. Secret names must be unique.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_secret_request** | [**CreateSecretRequest**](CreateSecretRequest.md) |  | [required] |

### Return type

[**models::CreateSecretResponse**](CreateSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_secret_handler

> delete_secret_handler(name)
Delete secret

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Secret name | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_secret_handler

> models::GetSecretResponse get_secret_handler(name)
Get secret

Get metadata for a secret. The secret value is never returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Secret name | [required] |

### Return type

[**models::GetSecretResponse**](GetSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_secrets_handler

> models::ListSecretsResponse list_secrets_handler()
List secrets

List all stored secrets. Only metadata (name, timestamps) is returned — secret values are never exposed.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListSecretsResponse**](ListSecretsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_secret_handler

> models::UpdateSecretResponse update_secret_handler(name, update_secret_request)
Update secret

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Secret name | [required] |
**update_secret_request** | [**UpdateSecretRequest**](UpdateSecretRequest.md) |  | [required] |

### Return type

[**models::UpdateSecretResponse**](UpdateSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

