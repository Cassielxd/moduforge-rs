#[macro_export]
macro_rules! node {
    ($name:expr) => {
        {
            let mut node = mf_core::node::Node::default();
            node.set_name($name);
            node
        }
    };
    ($name:expr, $desc:expr) => {
        {
            let mut node = mf_core::node::Node::default();
            node.set_name($name).set_desc($desc);
            node
        }
    };
    ($name:expr, $desc:expr, $content:expr) => {
        {
            let mut node = mf_core::node::Node::default();
            node.set_name($name).set_desc($desc).set_content($content);
            node
        }
    };
    ($name:expr, $desc:expr, $content:expr, $($key:expr => $value:expr),*) => {
        {
            use serde_json::Value;
            let mut node = mf_core::node::Node::default();
            node.set_name($name)
                .set_desc($desc)
                .set_content($content);
            $(
                node.set_attr($key, Some(Value::String($value.to_string())));
            )*
            node
        }
    };
}
