use std::env;

use super::{Provider, ProviderError};

pub struct EnvProvider;

impl Provider for EnvProvider {
    fn resolve<'a>(&'a self, key: &'a str) -> super::ResolveFuture<'a> {
        Box::pin(async move {
            env::var(key).map_err(|err| ProviderError::VariableNotFound {
                key: key.to_string(),
                cause_str: err.to_string(),
            })
        })
    }
}
