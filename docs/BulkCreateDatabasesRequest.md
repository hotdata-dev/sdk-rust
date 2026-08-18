# BulkCreateDatabasesRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **i64** | How many databases to create. | 
**default_catalog** | Option<**String**> | Name the default catalog answers to inside each database, as on a single create. Defaults to `default`. | [optional]
**default_schema** | Option<**String**> | Schema that unqualified table names resolve to inside each database. | [optional]
**expires_at** | Option<**String**> | When the created databases expire. Accepts an RFC 3339 timestamp or a relative duration such as `24h`, `90m`, or `7d`. | [optional]
**idempotency_key** | Option<**String**> | Repeat this value to retry a request safely. A retry carrying a key that was already used returns the original batch — the same `batch_id` and the same databases — instead of creating a second set.  The key identifies the request, not its contents: reusing a key with a different `count` or template returns the original batch unchanged rather than reporting a mismatch. Use a fresh key per distinct request. | [optional]
**name_template** | Option<**String**> | Optional display-label pattern for each database. `{index}` is replaced with the database's zero-based position — for example `tenant-{index}` produces `tenant-0`, `tenant-1`, and so on. Labels are not identifiers and are not required to be unique. | [optional]
**schemas** | Option<[**Vec<models::DatabaseDefaultSchemaDecl>**](DatabaseDefaultSchemaDecl.md)> | Schemas and tables to declare on every database in the batch, in the same shape a single create accepts. The declaration applies identically to each database, so a batch of 10,000 declaring one table yields 10,000 databases that each hold that table and are ready to load — with no follow-up call per database. Omitted or empty means each database starts with no tables. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


