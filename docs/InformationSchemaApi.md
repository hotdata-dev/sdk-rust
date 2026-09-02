# \InformationSchemaApi

All URIs are relative to *https://api.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**information_schema**](InformationSchemaApi.md#information_schema) | **GET** /v1/information_schema | List tables



## information_schema

> models::InformationSchemaResponse information_schema(connection_id, schema, table, include_columns, limit, cursor)
List tables

List discovered tables with optional filtering and pagination. Supports wildcard patterns (SQL %) for schema and table name filters. Set include_columns=true to include column definitions (omitted by default). Every table carries its declared storage layout — `partition_by` and `sorted_by` — which is fixed when the table is created and cannot be changed afterwards. Both are always present; an empty array means none was declared. Only tables in a Hotdata instant database declare a layout here, so a table discovered from an external connection always reports empty arrays.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**connection_id** | Option<**String**> | Filter by connection ID |  |
**schema** | Option<**String**> | Filter by schema name (supports % wildcards) |  |
**table** | Option<**String**> | Filter by table name (supports % wildcards) |  |
**include_columns** | Option<**bool**> | Include column definitions (default: false) |  |
**limit** | Option<**i32**> | Maximum number of tables per page |  |
**cursor** | Option<**String**> | Pagination cursor from a previous response |  |

### Return type

[**models::InformationSchemaResponse**](InformationSchemaResponse.md)

### Authorization

[WorkspaceId](../README.md#WorkspaceId), [BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

