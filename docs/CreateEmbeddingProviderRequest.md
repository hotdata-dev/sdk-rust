# CreateEmbeddingProviderRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key** | Option<**String**> | API key. Hotdata stores it as a secret for you. Cannot be combined with `secret_name`. | [optional]
**config** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Provider settings, such as `model`, `base_url` and `dimensions`. | [optional]
**name** | **String** |  | 
**provider_type** | **String** | Provider type. Only `service` (a hosted embedding API) is supported. | 
**secret_name** | Option<**String**> | Name of a stored secret that holds the API key. It is only sent to approved endpoints (by default, OpenAI's API); for other endpoints use `api_key`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


