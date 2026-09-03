# QueryRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | When true, execute the query asynchronously and return a query run ID for polling via GET /query-runs/{id}. The query results can be retrieved via GET /results/{id} once the query run status is \"succeeded\". | [optional][default to false]
**async_after_ms** | Option<**i32**> | If set (requires `async` = true), first attempt the query synchronously and wait up to this many milliseconds: if it finishes in time the full result is returned, otherwise an async response (a run id to poll) is returned. Must be at least 1000 and at most the server's configured maximum; a value out of that range, or set without `async` = true, is rejected with 400. | [optional]
**database_id** | Option<**String**> | Database to scope the query to (its id). Alternative to the `X-Database-Id` header — exactly one source must be provided. If both this field and the header are set and they disagree, the request is rejected with a 400. | [optional]
**default_catalog** | Option<**String**> | Catalog that unqualified table references resolve against within the query's database scope. Must name a catalog visible in the database (`default`, an attached catalog alias, or a system catalog). Defaults to `default` when omitted. | [optional]
**default_schema** | Option<**String**> | Schema that unqualified table references resolve against within the query's database scope.  Omit it to use the database's own default schema — the `default_schema` chosen when the database was created, or the single schema it declares, and only `main` when it has neither. The database's `default_schema` field (see `GET /v1/databases/{id}`) reports the value in force.  Setting this field overrides that per query, so sending `main` on a database whose data lives in another schema turns a query that works without the field into a \"table not found\" error. Existence is not validated up front — an unknown schema surfaces as a \"table not found\" error at planning time. Fully-qualified references (`<catalog>.<schema>.<table>`) are unaffected. | [optional]
**dialect** | Option<**String**> | SQL dialect the `sql` field is written in. One of `hotsql` (the default), `duckdb`, `postgres`, or `snowflake`. When set to anything other than `hotsql`, the query is translated to HotSQL before it runs, so you can use idioms from that dialect (for example Snowflake `IFF(...)` or Postgres `MOD(a, b)`). Only read-only queries are accepted. An unrecognized value is rejected with a 400. | [optional]
**sql** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


