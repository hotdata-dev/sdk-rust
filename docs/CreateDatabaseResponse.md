# CreateDatabaseResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**default_catalog** | **String** | Name the database's default catalog answers to inside its query scope (`default` unless overridden at create time). | 
**default_connection_id** | **String** | Internal id of the connection that backs this database's `default` catalog. Workspace-level connection endpoints (list, get, health, delete, cache purge) refuse to act on this id — it is exposed only for the managed-tables load endpoint (`POST /v1/connections/{id}/schemas/{s}/tables/{t}/loads`) so callers can publish parquet into tables declared at database-create time. Addressing it directly in SQL is not the recommended path — use `default` inside an `X-Database-Id` scope instead. | 
**default_schema** | **String** | Schema that unqualified table names resolve to inside this database's query scope. `main` unless the database declares a single schema or a `default_schema` was set at create time. | 
**expires_at** | Option<**String**> | When this database expires. | [optional]
**id** | **String** |  | 
**name** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


