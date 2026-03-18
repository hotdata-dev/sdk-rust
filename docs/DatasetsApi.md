# \DatasetsApi

All URIs are relative to *https://app.hotdata.dev*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_dataset**](DatasetsApi.md#create_dataset) | **POST** /v1/datasets | Create dataset
[**delete_dataset**](DatasetsApi.md#delete_dataset) | **DELETE** /v1/datasets/{id} | Delete dataset
[**get_dataset**](DatasetsApi.md#get_dataset) | **GET** /v1/datasets/{id} | Get dataset
[**list_dataset_versions**](DatasetsApi.md#list_dataset_versions) | **GET** /v1/datasets/{id}/versions | List dataset versions
[**list_datasets**](DatasetsApi.md#list_datasets) | **GET** /v1/datasets | List datasets
[**update_dataset**](DatasetsApi.md#update_dataset) | **PUT** /v1/datasets/{id} | Update dataset



## create_dataset

> models::CreateDatasetResponse create_dataset(create_dataset_request)
Create dataset

Create a new dataset from an uploaded file or inline data. The dataset becomes a queryable table under the `datasets` schema (e.g., `SELECT * FROM datasets.my_table`). Supports CSV, JSON, and Parquet formats. Optionally specify explicit column types.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_dataset_request** | [**CreateDatasetRequest**](CreateDatasetRequest.md) |  | [required] |

### Return type

[**models::CreateDatasetResponse**](CreateDatasetResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_dataset

> delete_dataset(id)
Delete dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Dataset ID | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dataset

> models::GetDatasetResponse get_dataset(id)
Get dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Dataset ID | [required] |

### Return type

[**models::GetDatasetResponse**](GetDatasetResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_dataset_versions

> models::ListDatasetVersionsResponse list_dataset_versions(id, limit, offset)
List dataset versions

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Dataset ID | [required] |
**limit** | Option<**i32**> | Maximum number of versions (default: 100, max: 1000) |  |
**offset** | Option<**i32**> | Pagination offset (default: 0) |  |

### Return type

[**models::ListDatasetVersionsResponse**](ListDatasetVersionsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_datasets

> models::ListDatasetsResponse list_datasets(limit, offset)
List datasets

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of datasets (default: 100, max: 1000) |  |
**offset** | Option<**i32**> | Pagination offset (default: 0) |  |

### Return type

[**models::ListDatasetsResponse**](ListDatasetsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_dataset

> models::UpdateDatasetResponse update_dataset(id, update_dataset_request)
Update dataset

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Dataset ID | [required] |
**update_dataset_request** | [**UpdateDatasetRequest**](UpdateDatasetRequest.md) |  | [required] |

### Return type

[**models::UpdateDatasetResponse**](UpdateDatasetResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

