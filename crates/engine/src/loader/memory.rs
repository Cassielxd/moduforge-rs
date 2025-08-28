use crate::loader::{DecisionLoader, LoaderError, LoaderResponse};
use crate::model::DecisionContent;
use ahash::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};

/// Loads decisions from in-memory hashmap
#[derive(Debug, Default)]
pub struct MemoryLoader {
    memory_refs: RwLock<HashMap<String, Arc<DecisionContent>>>,
}

impl MemoryLoader {
    pub fn add<K, D>(
        &self,
        key: K,
        content: D,
    ) -> Result<(), anyhow::Error>
    where
        K: Into<String>,
        D: Into<DecisionContent>,
    {
        let mut mref = self.memory_refs.write()
            .map_err(|_| anyhow::anyhow!("无法获取内存加载器写锁"))?;
        mref.insert(key.into(), Arc::new(content.into()));
        Ok(())
    }

    pub fn get<K>(
        &self,
        key: K,
    ) -> Result<Option<Arc<DecisionContent>>, anyhow::Error>
    where
        K: AsRef<str>,
    {
        let mref = self.memory_refs.read()
            .map_err(|_| anyhow::anyhow!("无法获取内存加载器读锁"))?;
        Ok(mref.get(key.as_ref()).cloned())
    }

    pub fn remove<K>(
        &self,
        key: K,
    ) -> Result<bool, anyhow::Error>
    where
        K: AsRef<str>,
    {
        let mut mref = self.memory_refs.write()
            .map_err(|_| anyhow::anyhow!("无法获取内存加载器写锁"))?;
        Ok(mref.remove(key.as_ref()).is_some())
    }
}

impl DecisionLoader for MemoryLoader {
    fn load<'a>(
        &'a self,
        key: &'a str,
    ) -> impl Future<Output = LoaderResponse> + 'a {
        async move {
            match self.get(key) {
                Ok(Some(content)) => Ok(content),
                Ok(None) => Err(LoaderError::NotFound(key.to_string()).into()),
                Err(e) => Err(LoaderError::Internal {
                    key: key.to_string(),
                    source: e,
                }.into()),
            }
        }
    }
}
