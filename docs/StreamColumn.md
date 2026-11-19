# StreamColumn

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from** | Option<[**models::StreamColumnSource**](StreamColumnSource.md)> |  | [optional]
**indexed** | Option<**bool**> | True for the `large_text` column that has a text index. Omitted when false. | [optional]
**name** | **String** | Column name. | 
**role** | **String** | `time`, `large_text`, `normal`, `map_key` or `extra`. | 
**r#type** | **String** | Column type: `string`, `int64`, `float64`, `boolean`, `timestamp(ns)`, `timestamp(us)`, `timestamp(ms)`, `timestamp(s)`, `list<T>` or `map<T>`. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


