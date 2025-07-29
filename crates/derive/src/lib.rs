//! ModuForge 派生宏
//!
//! 该模块提供了ModuForge项目的所有派生宏，包括：
//! - PState: 插件状态派生宏
//! - Component: 依赖注入组件派生宏
//! - Injectable: 可注入字段派生宏
//! - service: 服务标记宏
//! - bean: Bean方法标记宏

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
