# LoadManagedTableRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#async** | Option<**bool**> | Run the load in the background and return a job ID to poll. | [optional][default to false]
**async_after_ms** | Option<**i32**> | With `async: true`, wait up to this many milliseconds for the load before falling back to a job ID. Must be between 1000 and 30000. | [optional]
**columns** | Option<[**std::collections::HashMap<String, models::ColumnDefinition>**](ColumnDefinition.md)> | Column types for inline `data`, by column name. Must list every CSV column; omit to detect types from the data. Each value is a type name such as `BIGINT` or `DECIMAL(10,2)`, or an object such as `{\"type\": \"DECIMAL\", \"precision\": 10, \"scale\": 2}`.  Types: `VARCHAR`, `TEXT`, `STRING`, `CHAR`, `BOOLEAN`, `TINYINT`, `SMALLINT`, `INTEGER`, `BIGINT`, `UTINYINT`, `USMALLINT`, `UINTEGER`, `UBIGINT`, `REAL`, `FLOAT`, `DOUBLE`, `DECIMAL`, `NUMERIC`, `DATE`, `TIME`, `TIMESTAMP`, `TIMESTAMPTZ`, `BINARY`, `BLOB`, `UUID`, `JSON`. | [optional]
**data** | Option<**String**> | CSV text with a header row, up to 2 MiB. Larger data fails with 413 (`INLINE_DATA_TOO_LARGE`); upload it and use `upload_id` instead. | [optional]
**format** | Option<**String**> | File format of the upload: `csv`, `json` (one object per line) or `parquet`. Defaults to detection from the upload's content type, then its contents. Inline `data` accepts only `csv`; not allowed with `result_id`. | [optional]
**idempotency_key** | Option<**String**> | Your own key, such as a UUID, that makes a load of inline `data` safe to retry: a retry with the same key returns the first result and loads nothing. Up to 255 characters. Reusing a key for different data returns 409 `IDEMPOTENCY_KEY_CONFLICT`; a retry while the first attempt still runs returns a different 409, so wait and retry. | [optional]
**key** | Option<**Vec<String>**> | Key columns for `delete`, `update` and `upsert`, overriding the table's key. Use the same key on every load of a table. | [optional]
**mode** | **String** | How to apply the data. The first load into a table must be `replace`.  - `replace`: the data becomes the table's contents. - `append`: adds the rows. They must include every existing column (use   null for missing values); new columns are added. - `delete`: removes rows whose key matches a row in the data. - `update`: replaces rows whose key matches; other rows in the data are   ignored. - `upsert`: like `update`, but inserts rows whose key does not match.  `delete`, `update` and `upsert` need a key, from the table or from `key`. | 
**result_id** | Option<**String**> | ID of a saved query result. The rows are copied, so the table keeps them after the result expires. | [optional]
**upload_id** | Option<**String**> | ID of a finalized upload. Two loads cannot use the same upload at once; the second gets 409. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


