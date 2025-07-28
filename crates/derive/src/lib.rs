//! ModuForge 派生宏
//!
//! 该模块提供了ModuForge项目的所有派生宏，包括：
//! - PState: 插件状态派生宏
//! - Component: 依赖注入组件派生宏
//! - Injectable: 可注入字段派生宏
//! - service: 服务标记宏
//! - bean: Bean方法标记宏

mod di;

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

/// Component derive macro - 自动实现Component trait
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    di::derive_component(input)
}

/// Injectable derive macro - 标记需要依赖注入的字段
#[proc_macro_derive(Injectable, attributes(inject))]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    di::derive_injectable(input)
}

/// Service attribute macro - 标记服务组件
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    di::service(args, input)
}

/// Bean attribute macro - 标记Bean方法
#[proc_macro_attribute]
pub fn bean(args: TokenStream, input: TokenStream) -> TokenStream {
    di::bean(args, input)
}

/// BeforeAspect derive macro - 自动实现前置切面
#[proc_macro_derive(BeforeAspect, attributes(aspect))]
pub fn derive_before_aspect(input: TokenStream) -> TokenStream {
    di::derive_before_aspect(input)
}

/// AfterAspect derive macro - 自动实现后置切面
#[proc_macro_derive(AfterAspect, attributes(aspect))]
pub fn derive_after_aspect(input: TokenStream) -> TokenStream {
    di::derive_after_aspect(input)
}

/// AfterReturningAspect derive macro - 自动实现返回后切面
#[proc_macro_derive(AfterReturningAspect, attributes(aspect))]
pub fn derive_after_returning_aspect(input: TokenStream) -> TokenStream {
    di::derive_after_returning_aspect(input)
}

/// AfterThrowingAspect derive macro - 自动实现异常后切面  
#[proc_macro_derive(AfterThrowingAspect, attributes(aspect))]
pub fn derive_after_throwing_aspect(input: TokenStream) -> TokenStream {
    di::derive_after_throwing_aspect(input)
}

/// AroundAspect derive macro - 自动实现环绕切面
#[proc_macro_derive(AroundAspect, attributes(aspect))]
pub fn derive_around_aspect(input: TokenStream) -> TokenStream {
    di::derive_around_aspect(input)
}

/// AutoAop attribute macro - 为方法自动添加AOP拦截
#[proc_macro_attribute]
pub fn auto_aop(args: TokenStream, input: TokenStream) -> TokenStream {
    di::auto_aop(args, input)
}
