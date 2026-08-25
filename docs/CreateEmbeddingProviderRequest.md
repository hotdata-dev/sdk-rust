# CreateEmbeddingProviderRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key** | Option<**String**> | Inline API key. If provided, a secret is auto-created and referenced. Cannot be used together with `secret_name`. | [optional]
**config** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Provider-specific configuration (model name, base URL, dimensions, etc.) | [optional]
**name** | **String** |  | 
**provider_type** | **String** | Provider type: \"local\" or \"service\" | 
**secret_name** | Option<**String**> | Reference an existing stored secret by name (for service providers).  A stored secret is only sent to an approved provider origin — by default OpenAI's public API. To use a different endpoint, supply the key inline with `api_key` instead, or ask your operator to approve the origin. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


