# BulkCreateDatabasesRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **i64** | How many databases to create. | 
**default_catalog** | Option<**String**> | Name of each database's own catalog in SQL. Defaults to `default`. | [optional]
**default_schema** | Option<**String**> | Schema that unqualified table names resolve to in each database. | [optional]
**expires_at** | Option<**String**> | When the databases expire: a future RFC 3339 timestamp, or a duration such as `24h`, `90m` or `7d`. | [optional]
**idempotency_key** | Option<**String**> | Key that makes the request safe to retry. A repeated key returns the original batch, even if the rest of the request differs. | [optional]
**name_template** | Option<**String**> | Name pattern for the databases. Must contain `{index}`, which becomes each database's zero-padded position (`tenant-00` to `tenant-99` for a count of 100); a batch suffix is added to each name. | [optional]
**schemas** | Option<[**Vec<models::DatabaseDefaultSchemaDecl>**](DatabaseDefaultSchemaDecl.md)> | Schemas and tables to create in every database. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


