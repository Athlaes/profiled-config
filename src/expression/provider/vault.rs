use crate::expression::provider::{Provider, ResolveFuture};

pub struct VaultProvider;

impl Provider for VaultProvider {
    fn resolve<'a>(&'a self, key: &'a str) -> ResolveFuture<'a> {
        Box::pin(async move { Ok("".to_string()) })
    }
}
