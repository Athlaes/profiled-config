use crate::processor::ResolveError;

pub fn format_error(causes: &[ResolveError]) -> String {
    let mut formatted = String::new();
    for error in causes {
        formatted.push_str(format!("path: {} error: {}\n", error.path, error.cause).as_str());
    }
    formatted
}
