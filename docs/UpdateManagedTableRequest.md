# UpdateManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**constant_per_key** | Option<**Vec<String>**> | Columns whose value is the same for every row sharing this table's key.  Send `[]` to revoke the declaration, which restores the unrestricted search on the next load — this is the kill switch if a declaration turns out to be false.  **Correctness-affecting, not a hint.** If the assertion is false, a keyed mutation supersedes one version of a key and appends beside another, silently duplicating it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


