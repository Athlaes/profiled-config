pub const PROVIDERS: &[&str] = &["env"];
pub mod env;

pub trait Provider {
    // fn resolve(&self, key: &str) -> String;
}
