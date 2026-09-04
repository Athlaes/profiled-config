#[doc(hidden)]
pub use include_dir;
#[cfg(feature = "macros")]
pub use profiled_config_macros::profiled_config;

mod api;
mod error;
mod expression;
mod format;
mod merge;
mod pipeline;
mod source;

pub use error::ConfigError;
#[doc(hidden)]
pub use pipeline::load_config_from_dir;
