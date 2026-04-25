# Mustache template override

Fixes a limitation in openapi-generator's rust/reqwest target: the stock
templates model `Configuration.api_key` as a single `Option<ApiKey>` and
emit the same value for every `apiKey` security scheme on an operation.
For the Hotdata API — which declares distinct `X-Workspace-Id`,
`X-Sandbox-Id`, and `X-Session-Id` schemes — that means a sandbox id
would be sent as the workspace id and vice versa.

These templates replace that field with `api_keys: HashMap<String, ApiKey>`
keyed by header name, and change the per-operation header-emission code
to look up the right scheme: `configuration.api_keys.get("X-Workspace-Id")`.

## Why live here instead of upstream

Upstream PR [#19511](https://github.com/OpenAPITools/openapi-generator/pull/19511)
lands the same fix but was closed as breaking; the maintainer wants it
gated behind a generator flag (tracked in
[#20069](https://github.com/OpenAPITools/openapi-generator/pull/20069))
before merging. Until that gate work happens we ship the patch locally.

## Drift tripwire

The regenerate workflow runs `cargo check` on the generated client. If a
future openapi-generator release restructures the auth-emission code such
that our overrides produce non-compiling output, CI fails loudly rather
than shipping subtly wrong code.

The openapi-generator version is pinned in `openapitools.json`
(currently 7.20.0). Bumping that pin should be accompanied by diffing
these templates against upstream's `modules/openapi-generator/src/main/resources/rust/` directory.
