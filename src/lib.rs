#[doc(hidden)]
pub use include_dir;
#[doc(hidden)]
pub use tokio;

#[doc(hidden)]
pub use pipeline::load_config_from_dir_with;

mod api;
mod bootstrap;
mod error;
mod expression;
mod format;
mod merge;
mod pipeline;
mod source;

pub use api::LoadOptions;
pub use api::ProfiledConfigArgs;
pub use error::ConfigError;
pub use expression::provider::Provider;
pub use expression::provider::ProviderActivation;
pub use expression::provider::ProviderError;
pub use expression::provider::ProviderFactoryClosure;
pub use expression::provider::ProviderFactoryFuture;

#[cfg(feature = "macros")]
pub use profiled_config_macros::profiled_config;

#[cfg(feature = "auto-cli")]
#[doc(hidden)]
mod cli;
#[doc(hidden)]
#[cfg(feature = "auto-cli")]
pub use pipeline::load_config_from_dir;
