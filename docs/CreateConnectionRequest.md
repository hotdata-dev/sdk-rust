# CreateConnectionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**config** | **std::collections::HashMap<String, serde_json::Value>** | Connection settings. For `managed`, an optional `schemas` list declaring schemas and their tables. | 
**name** | **String** |  | 
**secret_id** | Option<**String**> | ID of a stored secret to authenticate with. A `managed` connection takes no credentials, so setting this fails with 400. | [optional]
**secret_name** | Option<**String**> | Name of a stored secret to authenticate with. A `managed` connection takes no credentials, so setting this fails with 400. | [optional]
**source_type** | **String** | Connection type. Only `managed` can be created: a catalog of tables you load data into. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


