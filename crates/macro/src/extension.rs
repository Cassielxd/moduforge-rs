/// Extension macro implementation for easier Extension creation
#[macro_export]
macro_rules! impl_extension {
    () => {
        {
            moduforge_core::extension::Extension::new()
        }
    };
    ($(attr:$attr:expr),*) => {
        {
            let mut ext = moduforge_core::extension::Extension::new();
            $(
                ext.add_global_attribute($attr);
            )*
            ext
        }
    };
    ($(plugin:$plugin:expr),*) => {
        {
            let mut ext = moduforge_core::extension::Extension::new();
            $(
                ext.add_plugin(std::sync::Arc::new($plugin));
            )*
            ext
        }
    };
    ($(op:$op:expr),*) => {
        {
            let mut ext = moduforge_core::extension::Extension::new();
            $(
                ext.add_op_fn(std::sync::Arc::new($op));
            )*
            ext
        }
    };
    ($(attr:$attr:expr),* ; $(plugin:$plugin:expr),*) => {
        {
            let mut ext = moduforge_core::extension::Extension::new();
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
            let mut ext = moduforge_core::extension::Extension::new();
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
