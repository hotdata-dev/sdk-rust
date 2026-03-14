# RefreshRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**connection_id** | Option<**String**> |  | [optional]
**data** | Option<**bool**> |  | [optional]
**include_uncached** | Option<**bool**> | Controls whether uncached tables are included in connection-wide data refresh.  - `false` (default): Only refresh tables that already have cached data.   This is the common case for keeping existing data up-to-date. - `true`: Also sync tables that haven't been cached yet, essentially performing   an initial sync for any new tables discovered since the connection was created.  This field only applies to connection-wide data refresh (when `data=true` and `table_name` is not specified). It has no effect on single-table refresh or schema refresh operations. | [optional]
**schema_name** | Option<**String**> |  | [optional]
**table_name** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


