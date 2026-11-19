# CreateStreamRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**columns** | Option<[**Vec<models::StreamColumnHint>**](StreamColumnHint.md)> | Columns to define up front, for fields the first rows may lack or whose type or role they cannot show. Other columns come from the first request. | [optional]
**retention_days** | Option<**i32**> | Days of rows to keep, 1 or more. Stored but not yet enforced. | [optional]
**schema** | Option<**String**> | Schema of the table. Defaults to the database's default schema. | [optional]
**table** | **String** | Table the stream writes to. It must not exist yet. | 
**time_column** | Option<**String**> | Row field that holds each row's time; every row must have it. Omit it and `time_unit` to have Hotdata add an `_ingested_at` column with the time each request arrived. | [optional]
**time_unit** | Option<**String**> | Format of `time_column`: `ns`, `us`, `ms` or `s` for an integer since the Unix epoch, or `rfc3339` for text. Required with `time_column`, and only allowed with it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


