use std::{env, pin::Pin};

use super::{Provider, ProviderError};

pub struct EnvProvider;

impl EnvProvider {
    pub fn create(_: ()) -> Pin<Box<dyn Future<Output = Result<Box<dyn Provider>, ProviderError>>>> {
        Box::pin(async { Ok(Box::new(Self) as Box<dyn Provider>) })
    }
}

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
