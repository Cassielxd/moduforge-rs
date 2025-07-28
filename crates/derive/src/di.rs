//! 依赖注入相关宏实现

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, ItemFn, ItemStruct, Lit,
    parse::Parse, parse::ParseStream, Token,
};

/// 解析属性参数的结构
#[derive(Default)]
struct ComponentArgs {
    name: Option<String>,
    lifecycle: Option<String>,
    concurrent_read: bool,
    async_lock: bool,
    profiles: Vec<String>,
    conditional: Option<String>,
    auto_proxy: bool,
}

/// 解析切面属性参数的结构
#[derive(Default)]
struct AspectArgs {
    name: Option<String>,
    type_pattern: Option<String>,
    method_pattern: Option<String>,
    priority: Option<i32>,
    pointcut: Option<String>,
}

impl Parse for ComponentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = ComponentArgs::default();
        
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            
            match ident.to_string().as_str() {
                "name" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.name = Some(s.value());
                    }
                }
                "lifecycle" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.lifecycle = Some(s.value());
                    }
                }
                "concurrent_read" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Bool(b) = lit {
                        args.concurrent_read = b.value();
                    }
                }
                "async_lock" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Bool(b) = lit {
                        args.async_lock = b.value();
                    }
                }
                "profiles" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.profiles = s.value().split(',').map(|s| s.trim().to_string()).collect();
                    }
                }
                "conditional" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.conditional = Some(s.value());
                    }
                }
                "auto_proxy" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Bool(b) = lit {
                        args.auto_proxy = b.value();
                    }
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "Unknown argument"));
                }
            }
            
            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }
        
        Ok(args)
    }
}

impl Parse for AspectArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = AspectArgs::default();
        
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            
            match ident.to_string().as_str() {
                "name" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.name = Some(s.value());
                    }
                }
                "type_pattern" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.type_pattern = Some(s.value());
                    }
                }
                "method_pattern" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.method_pattern = Some(s.value());
                    }
                }
                "priority" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Int(i) = lit {
                        args.priority = Some(i.base10_parse()?);
                    }
                }
                "pointcut" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        args.pointcut = Some(s.value());
                    }
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "Unknown aspect argument"));
                }
            }
            
            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }
        
        Ok(args)
    }
}

/// Component derive macro - 自动实现Component trait
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析component属性
    let mut args = ComponentArgs::default();
    for attr in &input.attrs {
        if attr.path().is_ident("component") {
            if let Ok(parsed_args) = attr.parse_args::<ComponentArgs>() {
                args = parsed_args;
                break;
            }
        }
    }
    
    let component_name = args.name.unwrap_or_else(|| format!("{}", name));
    let lifecycle = match args.lifecycle.as_deref() {
        Some("singleton") => quote! { ::mf_context::Lifecycle::Singleton },
        Some("transient") => quote! { ::mf_context::Lifecycle::Transient },
        Some("scoped") => quote! { ::mf_context::Lifecycle::Scoped },
        _ => quote! { ::mf_context::Lifecycle::Singleton },
    };
    let supports_concurrent_read = args.concurrent_read;
    let requires_async_lock = args.async_lock;
    let auto_proxy = args.auto_proxy;
    
    // 生成条件检查代码
    let condition_check = if !args.profiles.is_empty() {
        let profile_names: Vec<&String> = args.profiles.iter().collect();
        quote! {
            let profiles = vec![#(#profile_names),*];
            let profile_refs: Vec<&str> = profiles.iter().map(|s| s.as_str()).collect();
            if !::mf_context::profile::ProfileCondition::any_of(&profile_refs).matches() {
                return;
            }
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::mf_context::Component for #name #ty_generics #where_clause {
            fn component_name() -> &'static str {
                #component_name
            }
            
            fn lifecycle() -> ::mf_context::Lifecycle {
                #lifecycle
            }
        }
        
        #[automatically_derived]
        impl #impl_generics ::mf_context::MutableComponent for #name #ty_generics #where_clause {
            fn supports_concurrent_read() -> bool {
                #supports_concurrent_read
            }
            
            fn requires_async_lock() -> bool {
                #requires_async_lock
            }
        }
        
        #[::mf_context::ctor::ctor]
        fn #name() {
            #condition_check
            ::mf_context::registry::auto_register_component_with_auto_proxy::<#name>(#auto_proxy);
        }
    };
    
    TokenStream::from(expanded)
}

/// Injectable derive macro - 标记需要依赖注入的字段
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 收集需要注入的字段
    let mut inject_fields = Vec::new();
    
    if let syn::Data::Struct(data_struct) = &input.data {
        if let syn::Fields::Named(fields_named) = &data_struct.fields {
            for field in &fields_named.named {
                for attr in &field.attrs {
                    if attr.path().is_ident("inject") {
                        if let Some(field_name) = &field.ident {
                            inject_fields.push((field_name, &field.ty));
                        }
                    }
                }
            }
        }
    }
    
    // 生成构造函数
    let constructor_params = inject_fields.iter().map(|(name, ty)| {
        quote! { #name: #ty }
    });
    
    let constructor_assignments = inject_fields.iter().map(|(name, _)| {
        quote! { #name }
    });
    
    let other_fields = if let syn::Data::Struct(data_struct) = &input.data {
        if let syn::Fields::Named(fields_named) = &data_struct.fields {
            fields_named.named.iter()
                .filter(|field| {
                    !field.attrs.iter().any(|attr| attr.path().is_ident("inject"))
                })
                .map(|field| {
                    let field_name = &field.ident;
                    quote! { #field_name: Default::default() }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub async fn new_with_dependencies(
                #(#constructor_params),*
            ) -> ::mf_context::ContainerResult<Self> {
                Ok(Self {
                    #(#constructor_assignments,)*
                    #(#other_fields,)*
                })
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Service attribute macro - 标记服务组件
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = if args.is_empty() {
        ComponentArgs::default()
    } else {
        parse_macro_input!(args as ComponentArgs)
    };
    let input_struct = parse_macro_input!(input as ItemStruct);
    
    let name = &input_struct.ident;
    let component_name = args.name.unwrap_or_else(|| format!("{}", name));
    let lifecycle = match args.lifecycle.as_deref() {
        Some("singleton") => quote! { ::mf_context::Lifecycle::Singleton },
        Some("transient") => quote! { ::mf_context::Lifecycle::Transient },
        Some("scoped") => quote! { ::mf_context::Lifecycle::Scoped },
        _ => quote! { ::mf_context::Lifecycle::Singleton },
    };
    let supports_concurrent_read = args.concurrent_read;
    let requires_async_lock = args.async_lock;
    let auto_proxy = args.auto_proxy;
    
    // 生成条件检查代码
    let condition_check = if !args.profiles.is_empty() {
        let profile_names: Vec<&String> = args.profiles.iter().collect();
        quote! {
            let profiles = vec![#(#profile_names),*];
            let profile_refs: Vec<&str> = profiles.iter().map(|s| s.as_str()).collect();
            if !::mf_context::profile::ProfileCondition::any_of(&profile_refs).matches() {
                return;
            }
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        #input_struct
        
        impl ::mf_context::Component for #name {
            fn component_name() -> &'static str {
                #component_name
            }
            
            fn lifecycle() -> ::mf_context::Lifecycle {
                #lifecycle
            }
        }
        
        impl ::mf_context::MutableComponent for #name {
            fn supports_concurrent_read() -> bool {
                #supports_concurrent_read
            }
            
            fn requires_async_lock() -> bool {
                #requires_async_lock
            }
        }
        
        #[::mf_context::ctor::ctor]
        fn #name() {
            #condition_check
            ::mf_context::registry::auto_register_component_with_auto_proxy::<#name>(#auto_proxy);
        }
    };
    
    TokenStream::from(expanded)
}

/// Bean attribute macro - 标记Bean方法
pub fn bean(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = if args.is_empty() {
        ComponentArgs::default()
    } else {
        parse_macro_input!(args as ComponentArgs)
    };
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let bean_name = args.name.unwrap_or_else(|| format!("{}", fn_name));
    let lifecycle = match args.lifecycle.as_deref() {
        Some("singleton") => quote! { ::mf_context::Lifecycle::Singleton },
        Some("transient") => quote! { ::mf_context::Lifecycle::Transient },
        Some("scoped") => quote! { ::mf_context::Lifecycle::Scoped },
        _ => quote! { ::mf_context::Lifecycle::Singleton },
    };
    
    // 提取返回类型
    let return_type = match &input_fn.sig.output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        syn::ReturnType::Default => panic!("Bean functions must have a return type"),
    };
    
    let register_fn_name = syn::Ident::new(
        &format!("register_bean_{}", fn_name),
        Span::call_site(),
    );
    
    let expanded = quote! {
        #input_fn
        
        #[::mf_context::ctor::ctor]
        fn #register_fn_name() {
            ::mf_context::registry::register_bean_factory::<_, _, #return_type>(
                #bean_name,
                || Box::pin(async move {
                    let result = #fn_name().await;
                    Ok(std::sync::Arc::new(result) as std::sync::Arc<dyn std::any::Any + Send + Sync>)
                }),
                #lifecycle,
            );
        }
    };
    
    TokenStream::from(expanded)
}

/// BeforeAspect derive macro - 自动实现前置切面
pub fn derive_before_aspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析aspect属性
    let mut aspect_args = AspectArgs::default();
    for attr in &input.attrs {
        if attr.path().is_ident("aspect") {
            if let Ok(args) = attr.parse_args::<AspectArgs>() {
                aspect_args = args;
                break;
            }
        }
    }
    
    let aspect_name = aspect_args.name.unwrap_or_else(|| format!("{}", name));
    let priority = aspect_args.priority.unwrap_or(0);
    
    // 解析切点表达式
    let (type_pattern, method_pattern) = if let Some(pointcut) = aspect_args.pointcut {
        let parts: Vec<&str> = pointcut.split("::").collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("*".to_string(), "*".to_string())
        }
    } else {
        let type_pattern = aspect_args.type_pattern.unwrap_or_else(|| "*".to_string());
        let method_pattern = aspect_args.method_pattern.unwrap_or_else(|| "*".to_string());
        (type_pattern, method_pattern)
    };
    
    let register_fn_name = syn::Ident::new(
        &format!("register_before_aspect_{}", name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );
    
    let expanded = quote! {
        #[::mf_context::ctor::ctor]
        fn #register_fn_name() {
            let pointcut = ::mf_context::aop::Pointcut::new(#type_pattern, #method_pattern);
            let aspect = Box::new(<#name>::default()) as Box<dyn ::mf_context::aop::BeforeAspect>;
            ::mf_context::aop::add_before_aspect(pointcut, aspect);
        }
    };
    
    TokenStream::from(expanded)
}

/// AfterAspect derive macro - 自动实现后置切面
pub fn derive_after_aspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析aspect属性
    let mut aspect_args = AspectArgs::default();
    for attr in &input.attrs {
        if attr.path().is_ident("aspect") {
            if let Ok(args) = attr.parse_args::<AspectArgs>() {
                aspect_args = args;
                break;
            }
        }
    }
    
    let aspect_name = aspect_args.name.unwrap_or_else(|| format!("{}", name));
    let priority = aspect_args.priority.unwrap_or(0);
    
    // 解析切点表达式
    let (type_pattern, method_pattern) = if let Some(pointcut) = aspect_args.pointcut {
        let parts: Vec<&str> = pointcut.split("::").collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("*".to_string(), "*".to_string())
        }
    } else {
        let type_pattern = aspect_args.type_pattern.unwrap_or_else(|| "*".to_string());
        let method_pattern = aspect_args.method_pattern.unwrap_or_else(|| "*".to_string());
        (type_pattern, method_pattern)
    };
    
    let register_fn_name = syn::Ident::new(
        &format!("register_after_aspect_{}", name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );
    
    let expanded = quote! {
        #[::mf_context::ctor::ctor]
        fn #register_fn_name() {
            let pointcut = ::mf_context::aop::Pointcut::new(#type_pattern, #method_pattern);
            let aspect = Box::new(<#name>::default()) as Box<dyn ::mf_context::aop::AfterAspect>;
            ::mf_context::aop::add_after_aspect(pointcut, aspect);
        }
    };
    
    TokenStream::from(expanded)
}

/// AfterReturningAspect derive macro - 自动实现返回后切面
pub fn derive_after_returning_aspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析aspect属性
    let mut aspect_args = AspectArgs::default();
    for attr in &input.attrs {
        if attr.path().is_ident("aspect") {
            if let Ok(args) = attr.parse_args::<AspectArgs>() {
                aspect_args = args;
                break;
            }
        }
    }
    
    let aspect_name = aspect_args.name.unwrap_or_else(|| format!("{}", name));
    let priority = aspect_args.priority.unwrap_or(0);
    
    // 解析切点表达式
    let (type_pattern, method_pattern) = if let Some(pointcut) = aspect_args.pointcut {
        let parts: Vec<&str> = pointcut.split("::").collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("*".to_string(), "*".to_string())
        }
    } else {
        let type_pattern = aspect_args.type_pattern.unwrap_or_else(|| "*".to_string());
        let method_pattern = aspect_args.method_pattern.unwrap_or_else(|| "*".to_string());
        (type_pattern, method_pattern)
    };
    
    let register_fn_name = syn::Ident::new(
        &format!("register_after_returning_aspect_{}", name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );
    
    let expanded = quote! {
        #[::mf_context::ctor::ctor]
        fn #register_fn_name() {
            let pointcut = ::mf_context::aop::Pointcut::new(#type_pattern, #method_pattern);
            let aspect = Box::new(<#name>::default()) as Box<dyn ::mf_context::aop::AfterReturningAspect>;
            ::mf_context::aop::add_after_returning_aspect(pointcut, aspect);
        }
    };
    
    TokenStream::from(expanded)
}

/// AfterThrowingAspect derive macro - 自动实现异常后切面
pub fn derive_after_throwing_aspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析aspect属性  
    let mut aspect_args = AspectArgs::default();
    for attr in &input.attrs {
        if attr.path().is_ident("aspect") {
            if let Ok(args) = attr.parse_args::<AspectArgs>() {
                aspect_args = args;
                break;
            }
        }
    }
    
    let aspect_name = aspect_args.name.unwrap_or_else(|| format!("{}", name));
    let priority = aspect_args.priority.unwrap_or(0);
    
    // 解析切点表达式
    let (type_pattern, method_pattern) = if let Some(pointcut) = aspect_args.pointcut {
        let parts: Vec<&str> = pointcut.split("::").collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("*".to_string(), "*".to_string())
        }
    } else {
        let type_pattern = aspect_args.type_pattern.unwrap_or_else(|| "*".to_string());
        let method_pattern = aspect_args.method_pattern.unwrap_or_else(|| "*".to_string());
        (type_pattern, method_pattern)
    };
    
    let register_fn_name = syn::Ident::new(
        &format!("register_after_throwing_aspect_{}", name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );
    
    let expanded = quote! {
        #[::mf_context::ctor::ctor]
        fn #register_fn_name() {
            let pointcut = ::mf_context::aop::Pointcut::new(#type_pattern, #method_pattern);
            let aspect = Box::new(<#name>::default()) as Box<dyn ::mf_context::aop::AfterThrowingAspect>;
            ::mf_context::aop::add_after_throwing_aspect(pointcut, aspect);
        }
    };
    
    TokenStream::from(expanded)
}

/// AutoAop attribute macro - 为方法自动添加AOP拦截
pub fn auto_aop(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    
    // 提取方法的参数名
    let arg_names: Vec<_> = fn_sig.inputs.iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some(&pat_ident.ident)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
    // 构建参数字符串向量
    let args_vec = if arg_names.is_empty() {
        quote! { Vec::new() }
    } else {
        quote! { vec![#(format!("{:?}", #arg_names)),*] }
    };
    
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            let args = #args_vec;
            ::mf_context::aop::apply_aspects(
                std::any::type_name::<Self>(),
                stringify!(#fn_name),
                args,
                || async move #fn_block
            ).await
        }
    };
    
    TokenStream::from(expanded)
}

/// AopEnabled attribute macro - 为struct的方法自动添加AOP拦截  
pub fn aop_enabled(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let struct_vis = &input_struct.vis;
    let struct_attrs = &input_struct.attrs;
    let struct_fields = &input_struct.fields;
    let struct_generics = &input_struct.generics;
    
    let expanded = quote! {
        #(#struct_attrs)*
        #struct_vis struct #struct_name #struct_generics #struct_fields
    };
    
    TokenStream::from(expanded)
}

/// AroundAspect derive macro - 自动实现环绕切面
pub fn derive_around_aspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // 解析aspect属性
    let mut aspect_args = AspectArgs::default();
    for attr in &input.attrs {
        if attr.path().is_ident("aspect") {
            if let Ok(args) = attr.parse_args::<AspectArgs>() {
                aspect_args = args;
                break;
            }
        }
    }
    
    let aspect_name = aspect_args.name.unwrap_or_else(|| format!("{}", name));
    let priority = aspect_args.priority.unwrap_or(0);
    
    // 解析切点表达式
    let (type_pattern, method_pattern) = if let Some(pointcut) = aspect_args.pointcut {
        let parts: Vec<&str> = pointcut.split("::").collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("*".to_string(), "*".to_string())
        }
    } else {
        let type_pattern = aspect_args.type_pattern.unwrap_or_else(|| "*".to_string());
        let method_pattern = aspect_args.method_pattern.unwrap_or_else(|| "*".to_string());
        (type_pattern, method_pattern)
    };
    
    let register_fn_name = syn::Ident::new(
        &format!("register_around_aspect_{}", name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );
    
    let expanded = quote! {
        #[::mf_context::ctor::ctor]
        fn #register_fn_name() {
            let pointcut = ::mf_context::aop::Pointcut::new(#type_pattern, #method_pattern);
            let aspect = Box::new(<#name>::default()) as Box<dyn ::mf_context::aop::AroundAspect>;
            ::mf_context::aop::add_around_aspect(pointcut, aspect);
        }
    };
    
    TokenStream::from(expanded)
}