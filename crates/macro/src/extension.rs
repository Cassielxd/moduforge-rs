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
/// # 返回值
///
/// `init()` 方法返回 `Vec<mf_core::types::Extensions>`，这是一个包含所有扩展组件的数组：
/// - `Extensions::E(Extension)` - 扩展主体，包含操作函数、插件、全局属性等
/// - `Extensions::N(Node)` - 节点定义
/// - `Extensions::M(Mark)` - 标记定义
///
/// # 示例
///
/// ```rust
/// use mf_macro::{mf_extension, node, mark};
/// use mf_core::types::Extensions;
///
/// // 定义操作函数
/// fn setup_logging(_manager: &mf_state::ops::GlobalResourceManager) -> mf_core::ForgeResult<()> {
///     println!("日志系统初始化");
///     Ok(())
/// }
///
/// // 定义节点转换函数
/// fn node_transformer(node: &mf_core::node::Node) -> Option<mf_core::node::Node> {
///     if node.name() == "paragraph" && node.content().is_empty() {
///         let mut new_node = node.clone();
///         new_node.set_attr("placeholder", Some("输入文本..."));
///         Some(new_node)
///     } else {
///         Some(node.clone())
///     }
/// }
///
/// // 创建扩展
/// mf_extension!(
///     text_editor,
///     ops = [ setup_logging ],
///     node_transform = node_transformer,
///     nodes = [
///         node!("paragraph", "段落节点"),
///         node!("heading", "标题节点", "", "level" => "1")
///     ],
///     marks = [
///         mark!("bold", "粗体标记"),
///         mark!("italic", "斜体标记")
///     ],
///     docs = "文本编辑器扩展"
/// );
///
/// // 使用方法
/// let extensions = text_editor::init();
///
/// // 遍历所有扩展组件
/// for ext in &extensions {
///     match ext {
///         Extensions::E(extension) => println!("Extension 已加载"),
///         Extensions::N(node) => println!("节点: {}", node.name()),
///         Extensions::M(mark) => println!("标记: {}", mark.name()),
///     }
/// }
/// ```
///
/// ## 可用选项：
///
/// - `ops`: 操作函数列表，函数签名为 `fn(&GlobalResourceManager) -> ForgeResult<()>`
/// - `plugins`: 要包含的插件实例列表
/// - `global_attributes`: 全局属性项列表
/// - `node_transform`: 节点转换函数，签名为 `fn(&Node) -> Option<Node>`
/// - `nodes`: 节点定义列表，使用 node! 宏创建
/// - `marks`: 标记定义列表，使用 mark! 宏创建
/// - `docs`: 扩展的文档字符串
#[macro_export]
macro_rules! mf_extension {
    (
        $name:ident
        $(, ops = [ $( $op:ident ),+ $(,)? ] )?
        $(, plugins = [ $( $plugin:expr ),+ $(,)? ] )?
        $(, global_attributes = [ $( $attr:expr ),+ $(,)? ] )?
        $(, node_transform = $node_transform_fn:expr )?
        $(, nodes = [ $( $node:expr ),+ $(,)? ] )?
        $(, marks = [ $( $mark:expr ),+ $(,)? ] )?
        $(, docs = $docs:expr )?
        $(,)?
    ) => {
        $( #[doc = $docs] )?
        ///
        /// 用于框架的 ModuForge 扩展。
        /// 要使用它，请调用 init() 方法获取 Extensions 数组：
        ///
        /// ```rust,ignore
        /// use mf_core::types::Extensions;
        ///
        #[doc = concat!("let extensions = ", stringify!($name), "::init();")]
        /// ```
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl $name {
            /// 初始化此扩展以供 ModuForge 运行时使用。
            ///
            /// # 返回
            /// 返回一个 Extensions 枚举数组，包含：
            /// - Extension (作为 Extensions::E)
            /// - Node 定义 (作为 Extensions::N)
            /// - Mark 定义 (作为 Extensions::M)
            pub fn init() -> Vec<mf_core::types::Extensions> {
                let mut ext = mf_core::extension::Extension::new();

                // 添加操作函数
                $(
                    $(
                        let op_item = mf_core::extension::OpFnItem::new(std::sync::Arc::new($op));
                        ext.add_op_fn(op_item);
                    )+
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

                // 添加节点转换函数
                $(
                    let transform_fn = mf_core::extension::NodeTransformFn::new(
                        std::sync::Arc::new($node_transform_fn)
                    );
                    ext.add_node_transform(transform_fn);
                )?

                let mut extensions = Vec::new();

                // 添加 Extension 到数组
                extensions.push(mf_core::types::Extensions::E(ext));

                // 添加节点定义到数组
                $(
                    $(
                        extensions.push(mf_core::types::Extensions::N($node));
                    )+
                )?

                // 添加标记定义到数组
                $(
                    $(
                        extensions.push(mf_core::types::Extensions::M($mark));
                    )+
                )?

                extensions
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
        $(, nodes = [ $( $node:expr ),+ $(,)? ] )?
        $(, marks = [ $( $mark:expr ),+ $(,)? ] )?
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
            ///
            /// # 返回
            /// 返回一个 Extensions 枚举数组，包含：
            /// - Extension (作为 Extensions::E)
            /// - Node 定义 (作为 Extensions::N)
            /// - Mark 定义 (作为 Extensions::M)
            pub fn init( $( $config_field: $config_type ),+ ) -> Vec<mf_core::types::Extensions> {
                let mut ext = mf_core::extension::Extension::new();
                ($init_fn)(&mut ext, $( $config_field ),+ );

                let mut extensions = Vec::new();

                // 添加 Extension 到数组
                extensions.push(mf_core::types::Extensions::E(ext));

                // 添加节点定义到数组
                $(
                    $(
                        extensions.push(mf_core::types::Extensions::N($node));
                    )+
                )?

                // 添加标记定义到数组
                $(
                    $(
                        extensions.push(mf_core::types::Extensions::M($mark));
                    )+
                )?

                extensions
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
        attr_map.insert(
            $key.to_string(),
            AttributeSpec { default: Some(Value::String($value.to_string())) },
        );

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
        fn $name(
            _manager: &mf_state::ops::GlobalResourceManager
        ) -> mf_core::ForgeResult<()> {
            $body
        }
    };
    ($name:ident, |$manager:ident| $body:block) => {
        fn $name(
            $manager: &mf_state::ops::GlobalResourceManager
        ) -> mf_core::ForgeResult<()> {
            $body
        }
    };
}

/// 用于创建节点转换函数的辅助宏
#[macro_export]
macro_rules! mf_node_transform {
    ($name:ident, $body:block) => {
        fn $name(_node: &mf_core::node::Node) -> Option<mf_core::node::Node> {
            $body
        }
    };
    ($name:ident, |$node:ident| $body:block) => {
        fn $name($node: &mf_core::node::Node) -> Option<mf_core::node::Node> {
            $body
        }
    };
}
