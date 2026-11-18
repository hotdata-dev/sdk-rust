# CreateDatabaseRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**default_catalog** | Option<**String**> | Name of the database's own catalog in SQL. Defaults to `default`. Lowercase letters, digits and `_`, not starting with a digit. Some names are reserved, such as `hotdata` and `information_schema`. | [optional]
**default_schema** | Option<**String**> | Schema that unqualified table names resolve to. Defaults to the only schema in `schemas` if there is exactly one, otherwise `main`. | [optional]
**expires_at** | Option<**String**> | When the database expires: a future RFC 3339 timestamp, or a duration such as `90m`, `24h` or `7d`. Omit for no expiry. | [optional]
**if_not_exists** | Option<**bool**> | If a database with this `name` exists, return it with 200 instead of creating another. Requires `name`; the rest of the request is ignored on a match. | [optional]
**name** | Option<**String**> | Display name. Need not be unique. Defaults to a name based on the database ID. | [optional]
**schemas** | Option<[**Vec<models::DatabaseDefaultSchemaDecl>**](DatabaseDefaultSchemaDecl.md)> | Schemas and tables to create with the database. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


