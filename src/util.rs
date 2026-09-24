pub(crate) mod serialization;
pub(crate) mod web;

#[cfg(test)]
#[macro_export]
macro_rules! async_tests_with_env {
        ( $( async fn $name:ident() $(-> $ret:ty)? $body:block )+ ) => { $(
            #[test]
            fn $name() $(-> $ret)? {
                unsafe {
                    dotenvy::EnvLoader::new().load_and_modify().ok();
                }
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create Tokio runtime");
                runtime.block_on(async { $body })
            }
        )+ };
    }
