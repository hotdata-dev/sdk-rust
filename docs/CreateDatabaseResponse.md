# CreateDatabaseResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created** | Option<**bool**> | Whether this call brought the database into existence.  Only `false` when `if_not_exists` found a database already carrying the requested name, in which case nothing was created and the existing one is returned. The response status says the same thing — `201` against `200` — but generated clients often surface only the body, so it is stated here as well.  Always sent. It is declared optional so that a client built against a newer version of this API still accepts a response from a deployment that predates the field. Absent therefore means \"this deployment cannot say\", which is not the same as `false` — test for the two values explicitly rather than for truthiness. | [optional]
**default_catalog** | **String** | Name the database's default catalog answers to inside its query scope (`default` unless overridden at create time). | 
**default_connection_id** | **String** | Id of the connection that backs this database's `default` catalog. Two uses: pass it as `connection_id` to `POST /v1/databases/{other}/catalogs` to attach this database's catalog into another database, and as the connection in the managed-tables load endpoint (`POST /v1/connections/{id}/schemas/{s}/tables/{t}/loads`) to load tables declared at create time. Other connection endpoints (list, get, health, delete, cache purge) refuse to act on it. In SQL, address the catalog as `default` inside an `X-Database-Id` scope, not by this id. | 
**default_schema** | **String** | Schema that unqualified table names resolve to inside this database's query scope. `main` unless the database declares a single schema or a `default_schema` was set at create time. | 
**expires_at** | Option<**String**> | When this database expires. | [optional]
**forked_from** | Option<[**models::ForkedFromInfo**](ForkedFromInfo.md)> |  | [optional]
**id** | **String** |  | 
**name** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


