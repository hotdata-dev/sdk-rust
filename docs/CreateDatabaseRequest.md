# CreateDatabaseRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**default_catalog** | Option<**String**> | Optional name the database's auto-created default catalog answers to inside its query scope. Must be a valid SQL identifier (`[a-z0-9_]`, not starting with a digit) and may not collide with the reserved catalog names `hotdata`, `datasets`, or `information_schema`. Defaults to `default` when omitted, so `default.main.<table>` keeps working. | [optional]
**expires_at** | Option<**String**> | When this database expires. Accepts either an RFC 3339 timestamp (e.g. `\"2026-06-01T00:00:00Z\"`) or a relative duration suffixed with `h` (hours), `m` (minutes), or `d` (days) — for example `\"24h\"`, `\"48h\"`, or `\"7d\"`. Omitted (or empty) means the database never expires. Expiry is best-effort: the database will not be deleted before `expires_at`, but cleanup may run later than the exact timestamp. | [optional]
**name** | Option<**String**> | Optional free-form display label (for UIs/CLIs). Not unique. Not an identifier — databases are always addressed by `id`.  Accepts the legacy `description` key as an alias so clients that predate the rename keep populating this field. | [optional]
**schemas** | Option<[**Vec<models::DatabaseDefaultSchemaDecl>**](DatabaseDefaultSchemaDecl.md)> | Optional schemas/tables to declare on the database's auto-created default catalog. Tables declared here can be loaded via the standard managed-table load endpoint targeting `default_connection_id`. Omitted or empty means the default catalog starts empty. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


