use crate::{
    parser::ast::{ConfigValue, ConfigValueParts},
    provider::{self, Provider},
    selector::{self, Selector},
};

pub fn resolve(value: ConfigValue) -> String {
    let mut result = String::new();
    for part in &value.parts {
        match &part {
            ConfigValueParts::Literal(str) => {
                result.push_str(&str);
            }
            ConfigValueParts::Expression(exp) => {
                let provider = provider::get_provider(&exp.key);
                let value = provider.resolve(&exp.key);
                match value {
                    Ok(str) => {
                        if let Some(selector) = &exp.selector {
                            let selected_selector = selector::get_selector(selector.kind.as_str());
                            result.push_str(
                                &selected_selector
                                    .select(&str, &selector.query)
                                    .unwrap_or(exp.get_default_or_panic())
                                    .as_str(),
                            );
                        } else {
                            result.push_str(&str);
                        }
                    }
                    Err(err) => {
                        log::debug!("Couldn't resolve {} with error : {}", &exp.key, err);
                        result.push_str(&exp.get_default_or_panic());
                    }
                }
            }
        }
    }
    return result;
}
