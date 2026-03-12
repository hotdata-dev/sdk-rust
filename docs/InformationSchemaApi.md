# \InformationSchemaApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**information_schema**](InformationSchemaApi.md#information_schema) | **GET** /v1/information_schema | List tables



## information_schema

> models::InformationSchemaResponse information_schema(connection_id, schema, table, include_columns, limit, cursor)
List tables

List discovered tables with optional filtering and pagination. Supports wildcard patterns (SQL %) for schema and table name filters. Set include_columns=true to include column definitions (omitted by default).

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

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

