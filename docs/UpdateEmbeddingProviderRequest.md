# UpdateEmbeddingProviderRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**api_key** | Option<**String**> | New API key. Hotdata stores it as a secret for you. | [optional]
**config** | Option<**serde_json::Value**> |  | [optional]
**name** | Option<**String**> |  | [optional]
**secret_name** | Option<**String**> | Name of a stored secret that holds the API key. Send `null` to remove the key from the provider. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


