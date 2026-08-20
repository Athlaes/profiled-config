use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, PatType, parse_macro_input, spanned::Spanned};

use crate::args::ProfiledConfigArgs;

mod args;

#[proc_macro_attribute]
pub fn profiled_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attr as ProfiledConfigArgs);

    let function = parse_macro_input!(item as ItemFn);

    expand_profiled_config(function, attributes)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_profiled_config(function: ItemFn, args: ProfiledConfigArgs) -> syn::Result<proc_macro2::TokenStream> {
    if function.sig.ident != "main" {
        return Err(syn::Error::new(
            function.sig.ident.span(),
            "`profiled_config` must be applied to `main`",
        ));
    }

    let mut inputs = function.sig.inputs.iter();

    let config_argument = inputs.next().ok_or_else(|| {
        syn::Error::new(
            function.sig.inputs.span(),
            "`main` first argument should be a configuration argument",
        )
    })?;

    let FnArg::Typed(PatType {
        pat: config_pattern,
        ty: config_type,
        ..
    }) = config_argument
    else {
        return Err(syn::Error::new(
            config_argument.span(),
            "the configuration argument cannot be `self`",
        ));
    };

    let attributes = &function.attrs;
    let visibility = &function.vis;
    let output = &function.sig.output;
    let body = &function.block;

    let inner_name = format_ident!("__profiled_config_main");
    let is_async = function.sig.asyncness.is_some();

    let call = if is_async {
        quote! {
            #inner_name(config).await
        }
    } else {
        quote! {
            #inner_name(config)
        }
    };

    let asyncness = &function.sig.asyncness;

    let before_load = &args.before_load.map(|f| {
        quote! {
            #f();
        }
    });

    Ok(quote! {
        #asyncness fn #inner_name(
            #config_pattern: #config_type
        ) #output {
            #body
        }

        #(#attributes)*
        #visibility #asyncness fn main() #output {
            #before_load

            let config: #config_type = profiled_config::load_config!();

            #call
        }
    })
}
