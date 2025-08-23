/// 扩展宏实现，用于更简单的 Extension 创建（旧版）
#[macro_export]
macro_rules! impl_extension {
    () => {
        {
            mf_core::extension::Extension::new()
        }
    };
    ($(attr:$attr:expr),*) => {
        {
            let mut ext = mf_core::extension::Extension::new();
            $(
                ext.add_global_attribute($attr);
            )*
            ext
        }
    };
    ($(plugin:$plugin:expr),*) => {
        {
            let mut ext = mf_core::extension::Extension::new();
            $(
                ext.add_plugin(std::sync::Arc::new($plugin));
            )*
            ext
        }
    };
    ($(op:$op:expr),*) => {
        {
            let mut ext = mf_core::extension::Extension::new();
            $(
                ext.add_op_fn(std::sync::Arc::new($op));
            )*
            ext
        }
    };
    ($(attr:$attr:expr),* ; $(plugin:$plugin:expr),*) => {
        {
            let mut ext = mf_core::extension::Extension::new();
            $(
                ext.add_global_attribute($attr);
            )*
            $(
                ext.add_plugin(std::sync::Arc::new($plugin));
            )*
            ext
        }
    };
    ($(attr:$attr:expr),* ; $(plugin:$plugin:expr),* ; $(op:$op:expr),*) => {
        {
            let mut ext = mf_core::extension::Extension::new();
            $(
                ext.add_global_attribute($attr);
            )*
            $(
                ext.add_plugin(std::sync::Arc::new($plugin));
            )*
            $(
                ext.add_op_fn(std::sync::Arc::new($op));
            )*
            ext
        }
    };
}

/// 声明操作函数块。类似于 Deno 的 ops! 宏。
/// 
/// # 示例
/// 
/// ```rust
/// use std::sync::Arc;
/// use mf_core::{ForgeResult, extension::OpFn};
/// use mf_state::ops::GlobalResourceManager;
/// use mf_macro::mf_ops;
/// 
/// // 简单的操作块
/// mf_ops!(my_ops, [
///     op_hello,
///     op_world
/// ]);
/// 
/// fn op_hello(_manager: &GlobalResourceManager) -> ForgeResult<()> {
///     println!("Hello");
///     Ok(())
/// }
/// 
/// fn op_world(_manager: &GlobalResourceManager) -> ForgeResult<()> {
///     println!("World");
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! mf_ops {
    ($name:ident, [ $( $op:ident ),+ $(,)? ]) => {
        pub fn $name() -> mf_core::extension::OpFn {
            vec![
                $(
                    std::sync::Arc::new($op),
                )+
            ]
        }
    };
}

/// 定义具有声明式语法的 ModuForge 扩展，类似于 Deno 的 extension! 宏。
/// 此宏创建结构体和扩展的相关初始化方法。
/// 
/// # 示例
/// 
/// ```rust
/// use mf_macro::mf_extension;
/// use mf_core::types::GlobalAttributeItem;
/// use std::sync::Arc;
/// 
/// // 定义操作函数
/// fn setup_logging(_manager: &mf_state::ops::GlobalResourceManager) -> mf_core::ForgeResult<()> {
///     println!("日志系统初始化");
///     Ok(())
/// }
/// 
/// fn cleanup_resources(_manager: &mf_state::ops::GlobalResourceManager) -> mf_core::ForgeResult<()> {
///     println!("资源清理完成");
///     Ok(())
/// }
/// 
/// // 创建包含操作的扩展
/// mf_extension!(
///     logging_extension,
///     ops = [ setup_logging, cleanup_resources ],
///     docs = "用于日志记录和资源管理的扩展"
/// );
/// 
/// // 使用方法
/// let ext = logging_extension::init();
/// ```
/// 
/// ## 可用选项：
/// 
/// - `ops`: 操作函数列表，函数签名为 `fn(&GlobalResourceManager) -> ForgeResult<()>`
/// - `plugins`: 要包含的插件实例列表
/// - `global_attributes`: 全局属性项列表
/// - `docs`: 扩展的文档字符串
#[macro_export]
macro_rules! mf_extension {
    (
        $name:ident
        $(, ops = [ $( $op:ident ),+ $(,)? ] )?
        $(, plugins = [ $( $plugin:expr ),+ $(,)? ] )?
        $(, global_attributes = [ $( $attr:expr ),+ $(,)? ] )?
        $(, docs = $docs:expr )?
        $(,)?
    ) => {
        $( #[doc = $docs] )?
        /// 
        /// 用于框架的 ModuForge 扩展。
        /// 要使用它，请调用 init() 方法获取 Extension 实例：
        /// 
        /// ```rust,ignore
        /// use mf_core::extension::Extension;
        /// 
        #[doc = concat!("let extension = ", stringify!($name), "::init();")]
        /// ```
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl $name {
            /// 初始化此扩展以供 ModuForge 运行时使用。
            /// 
            /// # 返回
            /// 可在框架初始化期间使用的 Extension 对象
            pub fn init() -> mf_core::extension::Extension {
                let mut ext = mf_core::extension::Extension::new();
                
                // 添加操作函数
                $(
                    let ops: mf_core::extension::OpFn = vec![
                        $(
                            std::sync::Arc::new($op),
                        )+
                    ];
                    for op in ops {
                        ext.add_op_fn(op);
                    }
                )?

                // 添加插件
                $(
                    $(
                        ext.add_plugin(std::sync::Arc::new($plugin));
                    )+
                )?

                // 添加全局属性
                $(
                    $(
                        ext.add_global_attribute($attr);
                    )+
                )?
                
                ext
            }
        }
    };
}

/// 带配置支持的简化扩展宏
#[macro_export]
macro_rules! mf_extension_with_config {
    (
        $name:ident,
        config = { $( $config_field:ident : $config_type:ty ),+ $(,)? },
        init_fn = $init_fn:expr
        $(, docs = $docs:expr )?
        $(,)?
    ) => {
        $( #[doc = $docs] )?
        /// 
        /// 可配置的 ModuForge 扩展。
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl $name {
            /// 使用配置初始化此扩展。
            pub fn init( $( $config_field: $config_type ),+ ) -> mf_core::extension::Extension {
                let mut ext = mf_core::extension::Extension::new();
                ($init_fn)(&mut ext, $( $config_field ),+ );
                ext
            }
        }
    };
}

/// 用于创建全局属性项的辅助宏
#[macro_export]
macro_rules! mf_global_attr {
    ($types:expr, $attributes:expr) => {{
        use std::collections::HashMap;
        use mf_model::schema::AttributeSpec;
        
        let mut attr_map = HashMap::new();
        let attributes: Vec<(&str, AttributeSpec)> = $attributes;
        for (key, spec) in attributes {
            attr_map.insert(key.to_string(), spec);
        }
        
        mf_core::types::GlobalAttributeItem {
            types: $types.iter().map(|s| s.to_string()).collect(),
            attributes: attr_map,
        }
    }};
    
    // 用于字符串键值对的简化版本（创建基本的 AttributeSpec）
    ($type_name:expr, $key:expr, $value:expr) => {{
        use std::collections::HashMap;
        use mf_model::schema::AttributeSpec;
        use serde_json::Value;
        
        let mut attr_map = HashMap::new();
        attr_map.insert($key.to_string(), AttributeSpec {
            default: Some(Value::String($value.to_string())),
        });
        
        mf_core::types::GlobalAttributeItem {
            types: vec![$type_name.to_string()],
            attributes: attr_map,
        }
    }};
}

/// 用于创建带错误处理的操作函数的辅助宏
#[macro_export]
macro_rules! mf_op {
    ($name:ident, $body:block) => {
        fn $name(_manager: &mf_state::ops::GlobalResourceManager) -> mf_core::ForgeResult<()> {
            $body
        }
    };
    ($name:ident, |$manager:ident| $body:block) => {
        fn $name($manager: &mf_state::ops::GlobalResourceManager) -> mf_core::ForgeResult<()> {
            $body
        }
    };
}
