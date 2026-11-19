# ColumnTypeSpec

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**precision** | Option<**i32**> | Total digits for `DECIMAL` or `NUMERIC`, from 1 to 38. | [optional]
**scale** | Option<**i32**> | Digits after the decimal point for `DECIMAL` or `NUMERIC`. Requires `precision` and cannot exceed it; defaults to 0. | [optional]
**r#type** | **String** | Type name, such as `\"DECIMAL\"`, `\"TIMESTAMP\"` or `\"VARCHAR\"`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


