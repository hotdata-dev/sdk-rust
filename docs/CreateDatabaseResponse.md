# CreateDatabaseResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created** | Option<**bool**> | `true` if this call created the database; `false` if `if_not_exists` returned an existing one. | [optional]
**default_catalog** | **String** | Name of the database's own catalog in SQL. | 
**default_connection_id** | **String** | ID of the connection behind the database's own catalog. Use it to load tables through the connection endpoints, or to attach this catalog to another database. | 
**default_schema** | **String** | Schema that unqualified table names resolve to. | 
**expires_at** | Option<**String**> | When the database expires. | [optional]
**forked_from** | Option<[**models::ForkedFromInfo**](ForkedFromInfo.md)> |  | [optional]
**id** | **String** |  | 
**name** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


