# StreamColumnHint

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The row field the column holds. | 
**role** | Option<**String**> | `large_text` or `normal`. Large text is stored apart, so a query that does not read the column does not read its text. When omitted, the column's size in the first request decides it. | [optional]
**r#type** | Option<**String**> | The column's type: `string`, `int64`, `float64`, `boolean`, `timestamp(ns)`, `timestamp(us)`, `timestamp(ms)`, `timestamp(s)`, `list<T>` or `map<T>`, where `T` is one of these, nested at most 32 deep. A timestamp column reads integers in its unit, or RFC 3339 text. When omitted, the first request decides it. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


