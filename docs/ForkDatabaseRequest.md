# ForkDatabaseRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | Option<**String**> | Why the fork was taken, shown in the fork's lineage. Up to 4096 bytes. | [optional]
**expires_at** | Option<**String**> | When the fork expires: a future RFC 3339 timestamp, or a duration such as `24h` or `7d`. Defaults to the source's expiry, if it has one. | [optional]
**name** | Option<**String**> | Display name for the fork. Defaults to the source's name plus a short suffix. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


