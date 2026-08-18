# CreateEmbeddingProviderRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key** | Option<**String**> | Inline API key. If provided, a secret is auto-created and referenced. Cannot be used together with `secret_name`. | [optional]
**config** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Provider-specific configuration (model name, base URL, dimensions, etc.) | [optional]
**name** | **String** |  | 
**provider_type** | **String** | Provider type: \"local\" or \"service\" | 
**secret_name** | Option<**String**> | Reference an existing secret by name (for service providers). | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


