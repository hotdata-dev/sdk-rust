# RollupMeasure

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**centroids** | Option<**i64**> | For `approx_percentile` only: the summary's accuracy, as the `centroids` argument of `approx_percentile_cont`. Defaults to 100, at most 1000; a query uses the rollup only with the same value (100 when it omits one). | [optional]
**column** | Option<**String**> | Column to aggregate. Required for every kind except `count`, which counts all rows when it is omitted. | [optional]
**kind** | **Kind** | The aggregate a measure keeps.  - `count`: number of rows, or with a `column`, rows where it is not null. - `sum`, `min`, `max`: that aggregate of `column`. - `numeric`: `count`, `sum`, `min` and `max` of `column`, enough to answer   `count`, `sum`, `avg`, `min` and `max` queries on it. `avg` of a decimal   column is always read from the table. - `approx_percentile`: a summary of `column` that answers   `approx_percentile_cont` and `approx_median` queries at any percentile.   Answers are approximate and can differ slightly from the same query on the   table. Much larger than other measures, and grows with `centroids`. (enum: count, sum, min, max, numeric, approx_percentile) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


