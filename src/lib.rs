#![allow(unused_imports)]
// The following clippy lints are suppressed crate-wide because they fire only
// in openapi-generator stock-template output (src/apis/*, src/models/*) and
// would reappear on every regeneration. lib.rs is in .openapi-generator-ignore,
// so these suppressions survive regen. The hand-written ergonomic layer
// (auth.rs/arrow.rs/client.rs) is clean and does not rely on them.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_return)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::empty_docs)]

extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_repr;
extern crate url;

pub mod apis;
#[cfg(feature = "arrow")]
pub mod arrow;
pub mod auth;
pub mod client;
pub mod field;
pub mod models;
pub mod resources;

pub use apis::configuration::{ApiKey, BasicAuth, Configuration};
pub use apis::Error;
#[cfg(feature = "arrow")]
pub use arrow::{
    get_result_arrow, stream_result_arrow, ArrowBatchStream, ArrowError, ArrowResult,
    ARROW_STREAM_MEDIA_TYPE,
};
pub use auth::{BearerTokenProvider, TokenExchangeError, TokenManager};
#[cfg(feature = "arrow")]
pub use client::QueryToArrowError;
pub use client::{AwaitResultError, Client, ClientBuilder, ClientError, PollConfig};
pub use resources::{
    ConnectionTypesApi, ConnectionsApi, DatabaseContextApi, DatabasesApi, DatasetsApi,
    EmbeddingProvidersApi, IndexesApi, InformationSchemaApi, JobsApi, QueryApi, QueryRunsApi,
    RefreshApi, ResultsApi, SandboxesApi, SavedQueriesApi, SecretsApi, UploadsApi, WorkspacesApi,
};

pub mod prelude {
    pub use crate::apis::configuration::Configuration;
    #[cfg(feature = "arrow")]
    pub use crate::arrow::{ArrowError, ArrowResult};
    pub use crate::client::{Client, ClientBuilder, PollConfig};
    pub use crate::field;
    pub use crate::models::*;
    pub use crate::resources::*;
}
