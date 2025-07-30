//! ModuForge 派生宏
//!

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// 插件状态派生宏
#[proc_macro_derive(PState)]
pub fn derive_plugin_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Resource for #name {}
    };

    TokenStream::from(expanded)
}
