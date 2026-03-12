# CreateConnectionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**config** | Option<**serde_json::Value**> |  | 
**name** | **String** |  | 
**secret_id** | Option<**serde_json::Value**> | Optional reference to a secret by ID (e.g., \"secr_abc123\"). If provided, this secret will be used for authentication. Mutually exclusive with `secret_name`. | [optional]
**secret_name** | Option<**serde_json::Value**> | Optional reference to a secret by name. If provided, this secret will be used for authentication. Mutually exclusive with `secret_id`. | [optional]
**source_type** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


