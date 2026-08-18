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
                result.push_str(str);
            }
            ConfigValueParts::Expression(exp) => {
                let provider = provider::get_provider(&exp.provider);
                let value = provider.resolve(&exp.key);
                match value {
                    Ok(str) => {
                        if let Some(selector) = &exp.selector {
                            let selected_selector = selector::get_selector(selector.kind.as_str());
                            let selection_result = &selected_selector
                                .select(&str, &selector.query)
                                .unwrap_or_else(|err| {
                                    panic!(
                                        "Couldn't select value for {} with query {} : {}",
                                        exp.key, selector.query, err
                                    );
                                });
                            if selection_result.is_empty() {
                                result.push_str(&exp.get_default_or_panic());
                            } else {
                                result.push_str(selection_result);
                            }
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
    result
}
