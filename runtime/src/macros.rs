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

#[macro_export]
macro_rules! impl_plugin {
    ($name:ident, $append_fn:expr) => {
        #[derive(Debug)]
        pub struct $name {}

        #[async_trait]
        impl PluginTrait for $name where Self: Send + Sync {
            async fn append_transaction(
                &self,
                tr: &Transaction,
                old_state: &State,
                new_state: &State,
            ) -> Option<Transaction> {
                $append_fn(tr, old_state, new_state).await
            }

            async fn filter_transaction(
                &self,
                _tr: &Transaction,
                _state: &State,
            ) -> bool {
                true
            }

            async fn before_apply_transaction(
                &self,
                _tr: &mut Transaction,
                _state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }

            async fn after_apply_transaction(
                &self,
                _new_state: &State,
                _tr: &mut Transaction,
                _old_state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }
    };
    ($name:ident, $append_fn:expr, $filter_fn:expr) => {
        #[derive(Debug)]
        pub struct $name {}

        #[async_trait]
        impl PluginTrait for $name where Self: Send + Sync {
            async fn append_transaction(
                &self,
                tr: &Transaction,
                old_state: &State,
                new_state: &State,
            ) -> Option<Transaction> {
                $append_fn(tr, old_state, new_state).await
            }

            async fn filter_transaction(
                &self,
                tr: &Transaction,
                state: &State,
            ) -> bool {
                $filter_fn(tr, state)
            }

            async fn before_apply_transaction(
                &self,
                _tr: &mut Transaction,
                _state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }

            async fn after_apply_transaction(
                &self,
                _new_state: &State,
                _tr: &mut Transaction,
                _old_state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }
    };
    ($name:ident, $append_fn:expr, $filter_fn:expr, $before_fn:expr) => {
        #[derive(Debug)]
        pub struct $name {}

        #[async_trait]
        impl PluginTrait for $name where Self: Send + Sync {
            async fn append_transaction(
                &self,
                tr: &Transaction,
                old_state: &State,
                new_state: &State,
            ) -> Option<Transaction> {
                $append_fn(tr, old_state, new_state).await
            }

            async fn filter_transaction(
                &self,
                tr: &Transaction,
                state: &State,
            ) -> bool {
                $filter_fn(tr, state).await
            }

            async fn before_apply_transaction(
                &self,
                tr: &mut Transaction,
                state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                $before_fn(tr, state).await
            }

            async fn after_apply_transaction(
                &self,
                _new_state: &State,
                _tr: &mut Transaction,
                _old_state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }
    };
    ($name:ident, $append_fn:expr, $filter_fn:expr, $before_fn:expr, $after_fn:expr) => {
        #[derive(Debug)]
        pub struct $name {}

        #[async_trait]
        impl PluginTrait for $name where Self: Send + Sync {
            async fn append_transaction(
                &self,
                tr: &Transaction,
                old_state: &State,
                new_state: &State,
            ) -> Option<Transaction> {
                $append_fn(tr, old_state, new_state).await
            }

            async fn filter_transaction(
                &self,
                tr: &Transaction,
                state: &State,
            ) -> bool {
                $filter_fn(tr, state).await
            }

            async fn before_apply_transaction(
                &self,
                tr: &mut Transaction,
                state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                $before_fn(tr, state).await
            }

            async fn after_apply_transaction(
                &self,
                new_state: &State,
                tr: &mut Transaction,
                old_state: &State,
            ) -> Result<(), Box<dyn std::error::Error>> {
                $after_fn(new_state, tr, old_state).await
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
        impl StateField for $name where Self: Send + Sync {
            async fn init(
                &self,
                config: &StateConfig,
                instance: Option<&State>,
            ) -> PluginState {
                $init_fn(config, instance).await
            }

            async fn apply(
                &self,
                tr: &Transaction,
                value: PluginState,
                old_state: &State,
                new_state: &State,
            ) -> PluginState {
                $apply_fn(tr, value, old_state, new_state).await
            }
        }
    };
}

#[macro_export]
macro_rules! impl_command {
    ($name:ident, $execute_fn:expr) => {
        #[derive(Debug)]
        pub struct $name;

        #[async_trait]
        impl Command for $name {
            async fn execute(
                &self,
                tr: &mut Transaction,
            ) -> Result<(), TransformError> {
                $execute_fn(tr).await
            }

            fn name(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
    ($name:ident, $execute_fn:expr, $name_str:expr) => {
        #[derive(Debug)]
        pub struct $name;

        #[async_trait]
        impl Command for $name {
            async fn execute(
                &self,
                tr: &mut Transaction,
            ) -> Result<(), TransformError> {
                $execute_fn(tr).await
            }

            fn name(&self) -> String {
                $name_str.to_string()
            }
        }
    };
}
