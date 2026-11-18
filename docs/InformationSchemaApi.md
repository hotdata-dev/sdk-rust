# \InformationSchemaApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**information_schema**](InformationSchemaApi.md#information_schema) | **GET** /v1/information_schema | List tables



## information_schema

> models::InformationSchemaResponse information_schema(connection_id, schema, table, include_columns, limit, cursor)
List tables

List tables, with optional filters and pagination. `schema` and `table` accept SQL `%` wildcards. Set `include_columns=true` to include each table's columns.  Each table reports its `partition_by` and `sorted_by` layout, which is fixed when the table is created. An empty array means none was set. An API key limited to one database must pass that database's `connection_id`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | Option<**String**> | Filter by connection ID |  |
**schema** | Option<**String**> | Filter by schema name (supports % wildcards) |  |
**table** | Option<**String**> | Filter by table name (supports % wildcards) |  |
**include_columns** | Option<**bool**> | Include each table's columns. Default: false. |  |
**limit** | Option<**i32**> | Tables per page. Default 100, maximum 1000. |  |
**cursor** | Option<**String**> | The `next_cursor` from the previous page |  |

### Return type

[**models::InformationSchemaResponse**](InformationSchemaResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

