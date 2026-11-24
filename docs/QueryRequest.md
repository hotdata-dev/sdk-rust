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
**params** | Option<**serde_json::Value**> | Values for the parameters `sql` names as `$name`, by name. A value is bound as a value, never as SQL text, so it cannot change what the statement does. Works in reads and in `INSERT`, `UPDATE` and `DELETE`. Every `$name` in the SQL needs a value and every value must be used, or the request fails with 400.  Each value takes the type the SQL gives its parameter: `ts >= $start` reads `start` as a timestamp string, `id = $id` as the type of `id`, and a value that does not fit that type is refused. Where the SQL gives the parameter no type (`SELECT $x`), a string, number or boolean keeps its own type; write `CAST($x AS ...)` to choose another, and a null needs a typed position. Parameters are not available in other dialects, and cannot be arguments of `vector_search`, `bm25_search` or the vector distance functions. | [optional]
**sql** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


