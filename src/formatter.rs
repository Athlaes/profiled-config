use crate::resolver::ResolverError;

pub fn format_error(errors: Vec<ResolverError>) -> String {
    let mut formatted = String::new();
    for error in errors {
        formatted.push_str(format!("{}\n", error).as_str());
    }
    formatted
}
