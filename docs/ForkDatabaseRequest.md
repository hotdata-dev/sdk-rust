# ForkDatabaseRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**expires_at** | Option<**String**> | When the fork expires. Accepts either an RFC 3339 timestamp (e.g. `\"2026-06-01T00:00:00Z\"`) or a relative duration suffixed with `h` (hours), `m` (minutes), or `d` (days) — for example `\"24h\"` or `\"7d\"`. When omitted, a still-future expiry on the source is carried over; otherwise the fork never expires. | [optional]
**name** | Option<**String**> | Optional display label for the fork. When omitted, the source database's name (if any) is carried over. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


