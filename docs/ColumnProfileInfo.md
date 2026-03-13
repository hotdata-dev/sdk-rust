# ColumnProfileInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cardinality** | **i64** | Approximate number of distinct non-null values | 
**data_type** | **String** | Arrow data type (e.g. \"Utf8\", \"Int32\", \"Timestamp(Microsecond, Some(\\\"UTC\\\"))\") | 
**name** | **String** | Column name | 
**null_count** | **i64** | Number of null values | 
**null_percentage** | **f64** | Percentage of null values (0.0 to 100.0) | 
**profile** | Option<[**models::ColumnProfileDetail**](ColumnProfileDetail.md)> | Type-specific profile detail. Null when the column is all-null or has an unsupported type. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


