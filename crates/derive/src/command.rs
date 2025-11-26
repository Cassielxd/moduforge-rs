use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    FnArg, ItemFn, LitStr, Pat, PatIdent, Result, Token, Type, TypeReference,
};

pub struct CommandArgs {
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
            return Err(input.error("命令宏参数后仍然存在无法解析的内容"));
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
        return syn::Error::new(
            input_fn.sig.span(),
            "impl_command 只支持用于 async fn",
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

    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;

    let mut inputs = input_fn.sig.inputs.iter();
    let first = inputs.next();
    let mut invalid_first = true;
    if let Some(FnArg::Typed(arg)) = first {
        if matches!(&*arg.pat, Pat::Ident(pat) if pat.ident == "tr") {
            if let Type::Reference(TypeReference {
                mutability: Some(_), ..
            }) = &*arg.ty
            {
                invalid_first = false;
            }
        }
    }
    if invalid_first {
        return syn::Error::new(
            input_fn.sig.span(),
            "命令函数的第一个参数必须是 `tr: &mut Transaction`",
        )
        .to_compile_error()
        .into();
    }

    let extra_params: Vec<_> = inputs.cloned().collect();
    let (struct_generics, field_defs, ctor_params, ctor_inits, call_args) =
        match build_fields(&extra_params) {
            Ok(result) => result,
            Err(err) => return err.to_compile_error().into(),
        };

    let expanded = quote! {
        #input_fn

        #[derive(Debug)]
        #vis struct #command_struct #struct_generics {
            #(#field_defs),*
        }

        impl #struct_generics #command_struct #struct_generics {
            #vis fn new(#(#ctor_params),*) -> Self {
                Self { #(#ctor_inits),* }
            }
        }

        #[async_trait::async_trait]
        impl #struct_generics mf_state::transaction::CommandGeneric<
            mf_model::node_pool::NodePool,
            mf_model::schema::Schema
        > for #command_struct #struct_generics {
            async fn execute(
                &self,
                tr: &mut mf_state::Transaction,
            ) -> TransformResult<()> {
                #fn_name(tr, #(#call_args),*).await
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

fn build_fields(
    params: &[FnArg]
) -> Result<(
    TokenStream2,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
)> {
    if params.is_empty() {
        return Ok((quote! {}, Vec::new(), Vec::new(), Vec::new(), Vec::new()));
    }

    let lifetime = syn::Lifetime::new("'a", Span::call_site());
    let mut fields = Vec::new();
    let mut ctor_params = Vec::new();
    let mut ctor_inits = Vec::new();
    let mut call_args = Vec::new();

    for param in params {
        let arg = match param {
            FnArg::Typed(arg) => arg,
            _ => {
                return Err(syn::Error::new(
                    param.span(),
                    "命令函数的参数必须是标识符",
                ));
            },
        };

        let pat_ident = match &*arg.pat {
            Pat::Ident(PatIdent { ident, .. }) => ident,
            _ => {
                return Err(syn::Error::new(
                    arg.pat.span(),
                    "命令函数的参数必须是简单标识符",
                ));
            },
        };

        let ty_ref = match &*arg.ty {
            Type::Reference(TypeReference {
                mutability: None, elem, ..
            }) => TypeReference {
                and_token: Default::default(),
                lifetime: Some(lifetime.clone()),
                mutability: None,
                elem: elem.clone(),
            },
            _ => {
                return Err(syn::Error::new(
                    arg.ty.span(),
                    "除 `tr` 之外的参数必须是共享引用（`&T`）",
                ));
            },
        };

        let field_ty = Type::Reference(ty_ref);
        let field_def = quote! { pub #pat_ident: #field_ty };
        fields.push(field_def);

        ctor_params.push(quote! { #pat_ident: #field_ty });
        ctor_inits.push(quote! { #pat_ident });
        call_args.push(quote! { self.#pat_ident });
    }

    let generics = quote! { <'a> };

    Ok((generics, fields, ctor_params, ctor_inits, call_args))
}
