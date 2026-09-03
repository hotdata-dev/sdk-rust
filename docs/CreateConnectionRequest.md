# CreateConnectionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**config** | **std::collections::HashMap<String, serde_json::Value>** | Connection configuration object. Fields vary by source type (host, port, database, etc.). | 
**name** | **String** |  | 
**secret_id** | Option<**String**> | Optional reference to a secret by ID (e.g., \"secr_abc123\"). If provided, this secret will be used for authentication. Mutually exclusive with `secret_name`. | [optional]
**secret_name** | Option<**String**> | Optional reference to a secret by name. If provided, this secret will be used for authentication. Mutually exclusive with `secret_id`. | [optional]
**source_type** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


