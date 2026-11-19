# RollupBuildResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**job_id** | Option<**String**> | Background job that builds the rollup: poll `GET /v1/jobs/{job_id}` to follow it; the rollup becomes `ready` once the build catches up with the table. Null when the build could not be started: the rollup stays `building`, and rebuilding it starts the build. | [optional]
**rollup** | [**models::Rollup**](Rollup.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


