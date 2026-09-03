# ManagedTableConstantPerKeyResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connection_id** | **String** | Connection backing the catalog the table belongs to. For a database default catalog this is the database's `default_connection_id`, so it is the value that addresses the table through the connection-scoped endpoints — not the database id the request may have used. | 
**constant_per_key** | **Vec<String>** | The columns now declared constant per key. Empty means no declaration, i.e. the unrestricted search. | 
**schema** | **String** | Schema the table belongs to, as stored: lowercased, which may differ from the spelling in the request path. | 
**table** | **String** | Table the declaration was written to, as stored: lowercased, which may differ from the spelling in the request path. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


