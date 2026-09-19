# DatabaseDetailResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attachments** | [**Vec<models::DatabaseAttachmentInfo>**](DatabaseAttachmentInfo.md) |  | 
**created_at** | Option<**String**> | When the database was created. | [optional]
**default_catalog** | **String** | Name the database's default catalog answers to inside its query scope (`default` unless overridden at create time). | 
**default_connection_id** | **String** | Id of the connection backing this database's `default` catalog. Pass it as `connection_id` to `POST /v1/databases/{other}/catalogs` to attach this database's catalog into another database. In SQL, address the catalog as `default` inside an `X-Database-Id` scope, not by id. | 
**default_schema** | **String** | Schema that unqualified table names resolve to inside this database's query scope. `main` unless the database declares a single schema or a `default_schema` was set at create time. | 
**expires_at** | Option<**String**> | When this database expires. | [optional]
**forked_from** | Option<[**models::ForkedFromInfo**](ForkedFromInfo.md)> |  | [optional]
**id** | **String** |  | 
**name** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


