# QueryRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | Run the query in the background and return a query run ID right away. Poll `GET /v1/query-runs/{id}` for its status. | [optional][default to false]
**async_after_ms** | Option<**i32**> | With `async: true`, wait up to this many milliseconds for the result before falling back to a query run ID. Must be between 1000 and 30000. | [optional]
**database_id** | Option<**String**> | ID of the database to query. Use this or the `X-Database-Id` header; if both are set they must match, or the request fails with 400. | [optional]
**default_catalog** | Option<**String**> | Catalog that unqualified table names resolve to. Defaults to the database's default catalog; an unknown catalog fails with 400. | [optional]
**default_schema** | Option<**String**> | Schema that unqualified table names resolve to. Defaults to the database's `default_schema`; a schema that does not exist makes unqualified tables fail with \"table not found\". | [optional]
**dialect** | Option<**String**> | SQL dialect of `sql`: `hotsql` (default), `duckdb`, `postgres`, `snowflake` or `clickhouse`. Other dialects are translated to HotSQL before the query runs. | [optional]
**sql** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


