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
            async fn append_transaction(
                &self,
                tr: &Transaction,
                old_state: &State,
                new_state: &State,
            ) -> StateResult<Option<Transaction>> {
                $append_fn(tr, old_state, new_state).await
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

#[macro_export]
macro_rules! derive_plugin_state {
    ($name:ident) => {
        impl Resource for $name {}
    };
}
