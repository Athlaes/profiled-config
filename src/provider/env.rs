use crate::provider::Provider;

pub struct EnvProvider;

impl Provider for EnvProvider {
    fn resolve(&self, key: &str) -> Result<String, super::ProviderError> {
        todo!()
    }
}
