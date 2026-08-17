# ColumnTypeSpec

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**precision** | Option<**i32**> | Total number of digits for `DECIMAL` / `NUMERIC` (1–38). | [optional]
**scale** | Option<**i32**> | Number of digits after the decimal point for `DECIMAL` / `NUMERIC`. Requires `precision`, and cannot exceed it. | [optional]
**r#type** | **String** | The type name, e.g. `\"DECIMAL\"`, `\"TIMESTAMP\"`, `\"VARCHAR\"`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


