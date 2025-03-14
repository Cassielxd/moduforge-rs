
#[macro_export]
macro_rules! node {
    ($name:expr) => {
        {
            let mut node = $crate::node::Node::default();
            node.set_name($name);
            node
        }
    };
    ($name:expr, $desc:expr) => {
        {
            let mut node = $crate::node::Node::default();
            node.set_name($name).set_desc($desc);
            node
        }
    };
    ($name:expr, $desc:expr, $content:expr) => {
        {
            let mut node = $crate::node::Node::default();
            node.set_name($name).set_desc($desc).set_content($content);
            node
        }
    };
    ($name:expr, $desc:expr, $content:expr, $($key:expr => $value:expr),*) => {
        {
            let mut node = $crate::node::Node::default();
            node.set_name($name)
                .set_desc($desc)
                .set_content($content);
            $(
                node.set_attr($key, Some($value.to_string()));
            )*
            node
        }
    };
}

#[macro_export]
macro_rules! mark {
    ($name:expr) => {
        {
            let mut mark = $crate::mark::Mark::default();
            mark.set_name($name);
            mark
        }
    };
    ($name:expr, $desc:expr) => {
        {
            let mut mark = $crate::mark::Mark::default();
            mark.set_name($name).set_desc($desc);
            mark
        }
    };
    ($name:expr, $desc:expr, $($key:expr => $value:expr),*) => {
        {
            let mut mark = $crate::mark::Mark::default();
            mark.set_name($name)
                .set_desc($desc);
            $(
                mark.set_attr($key, Some($value));
            )*
            mark
        }
    };
} 