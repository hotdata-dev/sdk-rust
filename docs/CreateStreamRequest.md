# CreateStreamRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<[**Vec<models::StreamColumnHint>**](StreamColumnHint.md)> | Columns to fix before the first request: one the first rows may lack, or whose type or role those rows cannot show. Every other column still comes from the first request. | [optional]
**retention_days** | Option<**i32**> | How many days of rows to keep. Recorded, not yet enforced. | [optional]
**schema** | Option<**String**> | The table's schema. Defaults to the database's default schema. | [optional]
**table** | **String** | The table the stream writes. It must not exist yet. | 
**time_column** | Option<**String**> | The row field that holds each row's time. Every row must carry it. Leave it out, with `time_unit`, to have the server add a column `_ingested_at` instead: the time each request was received, in UTC to the microsecond, the same for every row of the request. | [optional]
**time_unit** | Option<**String**> | How the time is written: `ns`, `us`, `ms` or `s` for an integer count since 1970-01-01 UTC, or `rfc3339` for text such as `2026-11-06T17:21:00Z`. Required with `time_column`, and only with it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


