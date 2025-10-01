/// 传统插件实现宏（旧版）
/// 用于快速实现 PluginTrait，但缺少元数据支持
#[macro_export]
macro_rules! impl_plugin {
    ($name:ident, $append_fn:expr) => {
        #[derive(Debug)]
        pub struct $name {}

        #[async_trait]
        impl PluginTrait for $name
        where
            Self: Send + Sync,
        {
            fn metadata(&self) -> PluginMetadata {
                PluginMetadata {
                    name: stringify!($name).to_string(),
                    version: "1.0.0".to_string(),
                    description: "Auto-generated plugin".to_string(),
                    author: "Unknown".to_string(),
                    dependencies: vec![],
                    conflicts: vec![],
                    state_fields: vec![],
                    tags: vec![],
                }
            }

            async fn append_transaction(
                &self,
                trs: &[Transaction],
                old_state: &State,
                new_state: &State,
            ) -> StateResult<Option<Transaction>> {
                $append_fn(trs, old_state, new_state).await
            }

            async fn filter_transaction(
                &self,
                _tr: &Transaction,
                _state: &State,
            ) -> bool {
                true
            }
        }
    };
    ($name:ident, $append_fn:expr, $filter_fn:expr) => {
        #[derive(Debug)]
        pub struct $name {}

        #[async_trait]
        impl PluginTrait for $name
        where
            Self: Send + Sync,
        {
            fn metadata(&self) -> PluginMetadata {
                PluginMetadata {
                    name: stringify!($name).to_string(),
                    version: "1.0.0".to_string(),
                    description: "Auto-generated plugin".to_string(),
                    author: "Unknown".to_string(),
                    dependencies: vec![],
                    conflicts: vec![],
                    state_fields: vec![],
                    tags: vec![],
                }
            }

            async fn append_transaction(
                &self,
                trs: &[Transaction],
                old_state: &State,
                new_state: &State,
            ) -> StateResult<Option<Transaction>> {
                $append_fn(trs, old_state, new_state).await
            }

            async fn filter_transaction(
                &self,
                tr: &Transaction,
                state: &State,
            ) -> bool {
                $filter_fn(tr, state)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_state_field {
    ($name:ident, $init_fn:expr, $apply_fn:expr) => {
        #[derive(Debug)]
        pub struct $name;

        #[async_trait]
        impl StateField for $name
        where
            Self: Send + Sync,
        {
            async fn init(
                &self,
                config: &StateConfig,
                instance: &State,
            ) -> Arc<dyn Resource> {
                $init_fn(config, instance).await
            }

            async fn apply(
                &self,
                tr: &Transaction,
                value: Arc<dyn Resource>,
                old_state: &State,
                new_state: &State,
            ) -> Arc<dyn Resource> {
                $apply_fn(tr, value, old_state, new_state).await
            }
        }
    };
}

/// 创建插件元数据的宏，不需要名称参数（名称将由mf_plugin!宏自动提供）  
#[macro_export]
macro_rules! mf_meta {
    (
        version = $version:expr
        $(, description = $desc:expr)?
        $(, author = $author:expr)?
        $(, dependencies = [$($dep:expr),* $(,)?])?
        $(, conflicts = [$($conflict:expr),* $(,)?])?
        $(, state_fields = [$($field:expr),* $(,)?])?
        $(, tags = [$($tag:expr),* $(,)?])?
    ) => {{
        mf_state::plugin::PluginMetadata {
            name: "".to_string(),  // 将被mf_plugin!宏替换
            version: $version.to_string(),
            description: {
                #[allow(unused_mut)]
                let mut desc = "Auto-generated plugin".to_string();
                $(desc = $desc.to_string();)?
                desc
            },
            author: {
                #[allow(unused_mut)]
                let mut author = "Unknown".to_string();
                $(author = $author.to_string();)?
                author
            },
            dependencies: vec![$($($dep.to_string(),)*)?],
            conflicts: vec![$($($conflict.to_string(),)*)?],
            state_fields: vec![$($($field.to_string(),)*)?],
            tags: vec![$($($tag.to_string(),)*)?],
        }
    }};
}

/// 创建插件元数据的辅助宏 (已废弃，请使用 mf_meta!)
#[deprecated(
    since = "2.0.0",
    note = "请使用 mf_meta! 宏代替，它不需要重复指定插件名称"
)]
#[macro_export]
macro_rules! mf_plugin_metadata {
    ($name:expr) => {{
        mf_state::plugin::PluginMetadata {
            name: $name.to_string(),
            version: "1.0.0".to_string(),
            description: "Auto-generated plugin".to_string(),
            author: "Unknown".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }};

    ($name:expr, version = $version:expr) => {{
        mf_state::plugin::PluginMetadata {
            name: $name.to_string(),
            version: $version.to_string(),
            description: "Auto-generated plugin".to_string(),
            author: "Unknown".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }};

    ($name:expr,
     version = $version:expr,
     description = $desc:expr,
     author = $author:expr
     $(, dependencies = [$($dep:expr),* $(,)?])?
     $(, conflicts = [$($conflict:expr),* $(,)?])?
     $(, state_fields = [$($field:expr),* $(,)?])?
     $(, tags = [$($tag:expr),* $(,)?])?
    ) => {{
        mf_state::plugin::PluginMetadata {
            name: $name.to_string(),
            version: $version.to_string(),
            description: $desc.to_string(),
            author: $author.to_string(),
            dependencies: vec![$($($dep.to_string(),)*)?],
            conflicts: vec![$($($conflict.to_string(),)*)?],
            state_fields: vec![$($($field.to_string(),)*)?],
            tags: vec![$($($tag.to_string(),)*)?],
        }
    }};
}

/// 创建插件配置的辅助宏
#[macro_export]
macro_rules! mf_plugin_config {
    () => {{
        mf_state::plugin::PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        }
    }};

    (enabled = $enabled:expr, priority = $priority:expr) => {{
        mf_state::plugin::PluginConfig {
            enabled: $enabled,
            priority: $priority,
            settings: std::collections::HashMap::new(),
        }
    }};

    (enabled = $enabled:expr, priority = $priority:expr, settings = { $($key:expr => $value:expr),* $(,)? }) => {{
        let mut settings = std::collections::HashMap::new();
        $(
            settings.insert($key.to_string(), serde_json::json!($value));
        )*
        mf_state::plugin::PluginConfig {
            enabled: $enabled,
            priority: $priority,
            settings,
        }
    }};
}

/// 定义具有声明式语法的 ModuForge 插件，类似于 extension! 宏的设计
///
/// # 示例
///
/// ```rust
/// use mf_macro::{mf_plugin, mf_plugin_metadata, mf_plugin_config};
/// use mf_state::{Transaction, State, plugin::PluginMetadata, plugin::PluginConfig};
/// use mf_state::error::StateResult;
///
/// // 定义事务处理函数
/// async fn validate_transaction(
///     _trs: &[Transaction],
///     _old_state: &State,
///     _new_state: &State,
/// ) -> StateResult<Option<Transaction>> {
///     println!("验证事务");
///     Ok(None)
/// }
///
/// async fn filter_transaction(tr: &Transaction, _state: &State) -> bool {
///     // 简单的过滤逻辑
///     true
/// }
///
/// // 使用声明式语法创建插件
/// mf_plugin!(
///     validation_plugin,
///     metadata = mf_plugin_metadata!(
///         "validation_plugin",
///         version = "1.0.0",
///         description = "事务验证插件",
///         author = "ModuForge Team",
///         tags = ["validation", "security"]
///     ),
///     config = mf_plugin_config!(
///         enabled = true,
///         priority = 10,
///         settings = { "strict_mode" => true, "timeout" => 5000 }
///     ),
///     append_transaction = validate_transaction,
///     filter_transaction = filter_transaction,
///     docs = "用于事务验证和安全检查的插件"
/// );
///
/// // 使用方法
/// let plugin = validation_plugin::new();
/// let spec = validation_plugin::spec();
/// ```
#[macro_export]
macro_rules! mf_plugin {
    (
        $name:ident
        $(, metadata = $metadata:expr)?
        $(, config = $config:expr)?
        $(, append_transaction = $append_fn:expr)?
        $(, filter_transaction = $filter_fn:expr)?
        $(, state_field = $state_field:expr)?
        $(, docs = $docs:expr)?
        $(,)?
    ) => {
        $( #[doc = $docs] )?
        ///
        /// 用于框架的 ModuForge 插件。
        /// 要使用它，请调用 new() 方法获取插件实例或 spec() 方法获取插件规范：
        ///
        /// ```rust,ignore
        /// use mf_state::plugin::{Plugin, PluginSpec};
        ///
        #[doc = concat!("let plugin = ", stringify!($name), "::new();")]
        #[doc = concat!("let spec = ", stringify!($name), "::spec();")]
        /// ```
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建插件实例
            pub fn new() -> mf_state::plugin::Plugin {
                let spec = Self::spec();
                mf_state::plugin::Plugin::new(spec)
            }

            /// 获取插件规范
            pub fn spec() -> mf_state::plugin::PluginSpec {
                let trait_impl = std::sync::Arc::new(Self);
                mf_state::plugin::PluginSpec {
                    state_field: {
                        #[allow(unused_mut)]
                        let mut field: Option<std::sync::Arc<dyn mf_state::plugin::ErasedStateField>> = None;
                        $(
                            field = Some(std::sync::Arc::new($state_field) as std::sync::Arc<dyn mf_state::plugin::ErasedStateField>);
                        )?
                        field
                    },
                    tr: trait_impl,
                }
            }
        }

        #[async_trait::async_trait]
        impl mf_state::plugin::PluginTrait for $name {
            fn metadata(&self) -> mf_state::plugin::PluginMetadata {
                #[allow(unreachable_code)]
                {
                    $(
                        let mut metadata = $metadata;
                        metadata.name = stringify!($name).to_string();
                        return metadata;
                    )?
                    mf_state::plugin::PluginMetadata {
                        name: stringify!($name).to_string(),
                        version: "1.0.0".to_string(),
                        description: "Auto-generated plugin".to_string(),
                        author: "Unknown".to_string(),
                        dependencies: vec![],
                        conflicts: vec![],
                        state_fields: vec![],
                        tags: vec![],
                    }
                }
            }

            $(
                fn config(&self) -> mf_state::plugin::PluginConfig {
                    $config
                }
            )?

            $(
                async fn append_transaction(
                    &self,
                    trs: &[std::sync::Arc<mf_state::transaction::Transaction>],
                    old_state: &std::sync::Arc<mf_state::state::State>,
                    new_state: &std::sync::Arc<mf_state::state::State>,
                ) -> mf_state::error::StateResult<Option<mf_state::transaction::Transaction>> {
                    ($append_fn)(trs, old_state, new_state).await
                }
            )?

            $(
                async fn filter_transaction(
                    &self,
                    tr: &mf_state::transaction::Transaction,
                    state: &mf_state::state::State,
                ) -> bool {
                    ($filter_fn)(tr, state).await
                }
            )?
        }

    };
}

/// 带配置支持的可配置插件宏
#[macro_export]
macro_rules! mf_plugin_with_config {
    (
        $name:ident,
        config = { $( $config_field:ident : $config_type:ty ),+ $(,)? },
        init_fn = $init_fn:expr
        $(, docs = $docs:expr )?
        $(,)?
    ) => {
        $( #[doc = $docs] )?
        ///
        /// 可配置的 ModuForge 插件。
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 使用配置创建插件实例
            pub fn new( $( $config_field: $config_type ),+ ) -> mf_state::plugin::Plugin {
                let spec = Self::spec($( $config_field ),+);
                mf_state::plugin::Plugin::new(spec)
            }

            /// 使用配置获取插件规范
            pub fn spec( $( $config_field: $config_type ),+ ) -> mf_state::plugin::PluginSpec {
                ($init_fn)($( $config_field ),+)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_plugin_state {
    ($name:ident) => {
        impl Resource for $name {}
    };
}
