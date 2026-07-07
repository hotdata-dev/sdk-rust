# RefreshRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | When true, submit the refresh as a background job and return immediately with a job ID for status polling. Only supported for data refresh operations. | [optional]
**async_after_ms** | Option<**i32**> | If set (requires `async` = true), wait up to this many milliseconds for the refresh to finish: if it completes in time the full result is returned, otherwise a `202` with a job ID to poll. Must be between 1000 and the server maximum; a value out of that range, or set without `async` = true, is rejected with 400. Only applies to data refresh. | [optional]
**connection_id** | Option<**String**> |  | [optional]
**data** | Option<**bool**> |  | [optional]
**include_uncached** | Option<**bool**> | Controls whether uncached tables are included in connection-wide data refresh.  - `false` (default): Only refresh tables that already have cached data.   This is the common case for keeping existing data up-to-date. - `true`: Also sync tables that haven't been cached yet, essentially performing   an initial sync for any new tables discovered since the connection was created.  This field only applies to connection-wide data refresh (when `data=true` and `table_name` is not specified). It has no effect on single-table refresh or schema refresh operations. | [optional]
**schema_name** | Option<**String**> |  | [optional]
**table_name** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


