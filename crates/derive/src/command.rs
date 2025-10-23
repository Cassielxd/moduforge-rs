use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Error, ItemFn, LitStr, Result, Token,
};

struct CommandArgs {
    ident: Ident,
    command_name: Option<LitStr>,
}

impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let command_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse::<LitStr>()?)
        } else {
            None
        };

        if !input.is_empty() {
            return Err(
                input.error("unexpected tokens after command macro arguments")
            );
        }

        Ok(Self { ident, command_name })
    }
}
pub fn impl_command(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.asyncness.is_none() {
        return Error::new(
            input_fn.sig.span(),
            "impl_command 只能用于 async fn",
        )
        .to_compile_error()
        .into();
    }

    let attr_ts: TokenStream2 = attr.into();
    let (command_struct, command_name_lit) =
        match parse_args(attr_ts, &input_fn.sig.ident) {
            Ok(pair) => pair,
            Err(err) => return err.to_compile_error().into(),
        };

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;

    let expanded = quote! {
        #input_fn

        #[derive(Debug)]
        #vis struct #command_struct;

        #[async_trait::async_trait]
        impl Command for #command_struct {
            async fn execute(
                &self,
                tr: &mut Transaction,
            ) -> TransformResult<()> {
                #fn_name(tr).await
            }

            fn name(&self) -> String {
                #command_name_lit.to_string()
            }
        }
    };

    expanded.into()
}

fn parse_args(
    attr: TokenStream2,
    fn_ident: &Ident,
) -> Result<(Ident, LitStr)> {
    if attr.is_empty() {
        let default_ident = default_struct_ident(fn_ident);
        let lit = LitStr::new(&default_ident.to_string(), Span::call_site());
        return Ok((default_ident, lit));
    }

    let args = syn::parse2::<CommandArgs>(attr)?;
    let cmd_name = args.command_name.unwrap_or_else(|| {
        LitStr::new(&args.ident.to_string(), args.ident.span())
    });
    Ok((args.ident, cmd_name))
}

fn default_struct_ident(fn_ident: &Ident) -> Ident {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in fn_ident.to_string().chars() {
        if ch == '_' {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            for upper in ch.to_uppercase() {
                result.push(upper);
            }
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result.push_str("Command");
    format_ident!("{}", result)
}
