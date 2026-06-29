# MintedUploadPartResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**part_number** | **i32** | The 1-based part number this URL is for. | 
**url** | **String** | Short-lived URL to `PUT` this part's bytes to. Keep the response's `ETag` and pass the `{part_number, e_tag}` pair to finalize. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


