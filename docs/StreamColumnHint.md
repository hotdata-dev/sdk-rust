# StreamColumnHint

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | Row field the column holds. | 
**role** | Option<**String**> | `large_text` or `normal`. A `large_text` column is stored separately, so queries that do not use it do not read it. When omitted, the first request decides it. | [optional]
**r#type** | Option<**String**> | Column type: `string`, `int64`, `float64`, `boolean`, `timestamp(ns)`, `timestamp(us)`, `timestamp(ms)`, `timestamp(s)`, `list<T>` or `map<T>`. When omitted, the first request decides it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


