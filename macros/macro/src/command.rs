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
            ) -> TransformResult<()> {
                $execute_fn(tr).await
            }

            fn name(&self) -> String {
                $name_str.to_string()
            }
        }
    };
}
