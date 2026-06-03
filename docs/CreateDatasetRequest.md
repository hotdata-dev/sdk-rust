# CreateDatasetRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**label** | **String** |  | 
**source** | [**models::DatasetSource**](DatasetSource.md) |  | 
**storage_backend** | Option<**String**> | Optional storage backend: `\"parquet\"` (default) or `\"ducklake\"`. `\"ducklake\"` requires `ducklake.metadata_pg_url` to be configured at engine boot; the engine also rejects the combo of `storage_backend: \"ducklake\"` with a saved-query source or with explicit geometry columns (both deferred to a follow-up). | [optional]
**table_name** | Option<**String**> | Optional table_name - if not provided, derived from label | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


