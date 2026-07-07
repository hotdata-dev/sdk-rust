# QueryRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | When true, execute the query asynchronously and return a query run ID for polling via GET /query-runs/{id}. The query results can be retrieved via GET /results/{id} once the query run status is \"succeeded\". | [optional]
**async_after_ms** | Option<**i32**> | If set (requires `async` = true), first attempt the query synchronously and wait up to this many milliseconds: if it finishes in time the full result is returned, otherwise an async response (a run id to poll) is returned. Must be at least 1000 and at most the server's configured maximum; a value out of that range, or set without `async` = true, is rejected with 400. | [optional]
**database_id** | Option<**String**> | Database to scope the query to (its id). Alternative to the `X-Database-Id` header — exactly one source must be provided. If both this field and the header are set and they disagree, the request is rejected with a 400. | [optional]
**default_catalog** | Option<**String**> | Catalog that unqualified table references resolve against within the query's database scope. Must name a catalog visible in the database (`default`, an attached catalog alias, or a system catalog). Defaults to `default` when omitted. | [optional]
**default_schema** | Option<**String**> | Schema that unqualified table references resolve against within the query's database scope. Defaults to `main` when omitted. Existence is not validated up front — an unknown schema surfaces as a \"table not found\" error at planning time. | [optional]
**sql** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


