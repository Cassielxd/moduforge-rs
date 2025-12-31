#[macro_export]
macro_rules! mark {
    ($name:expr) => {
        {
            let mut mark = mf_core::mark::Mark::default();
            mark.set_name($name);
            mark
        }
    };
    ($name:expr, $desc:expr) => {
        {
            let mut mark = mf_core::mark::Mark::default();
            mark.set_name($name).set_desc($desc);
            mark
        }
    };
    ($name:expr, $desc:expr, $($key:expr => $value:expr),*) => {
        {
            use serde_json::Value;
            let mut mark = mf_core::mark::Mark::default();
            mark.set_name($name)
                .set_desc($desc);
            $(
                mark.set_attr($key, Some(Value::String($value.to_string())));
            )*
            mark
        }
    };
}
