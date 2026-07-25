use syn::{
    ExprPath, Token,
    parse::{Parse, ParseStream},
};

pub struct ProfiledConfigArgs {
    pub before_load: Option<ExprPath>,
}

impl Parse for ProfiledConfigArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { before_load: None });
        }

        let option: syn::Ident = input.parse()?;

        if option != "before_load" {
            return Err(syn::Error::new(option.span(), "expected `before_load`"));
        }

        input.parse::<Token![=]>()?;

        let before_load = input.parse::<ExprPath>()?;

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after `before_load`"));
        }

        Ok(Self {
            before_load: Some(before_load),
        })
    }
}
