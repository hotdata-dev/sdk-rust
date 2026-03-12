# ColumnTypeSpec

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**geometry_type** | Option<**serde_json::Value**> | Geometry type for GEOMETRY/GEOGRAPHY columns. E.g., \"Point\", \"LineString\", \"Polygon\", \"MultiPoint\", \"MultiLineString\", \"MultiPolygon\", \"GeometryCollection\", or \"Geometry\" (any). | [optional]
**precision** | Option<**serde_json::Value**> | Precision for DECIMAL type (1-38) | [optional]
**scale** | Option<**serde_json::Value**> | Scale for DECIMAL type | [optional]
**srid** | Option<**serde_json::Value**> | Spatial Reference System Identifier for GEOMETRY/GEOGRAPHY types. Common values: 4326 (WGS84), 3857 (Web Mercator). | [optional]
**r#type** | **String** | The data type name (e.g., \"DECIMAL\", \"TIMESTAMP\", \"GEOMETRY\") | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


