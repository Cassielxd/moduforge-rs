use std::sync::Arc;
use tokio::sync::RwLock;

pub mod client;
pub mod conn;
pub mod mapping;
pub mod mapping_v2;
pub mod provider;
pub mod types;
pub mod utils;

pub type ClientResult<T> = anyhow::Result<T>;
pub mod yrs {
    pub use yrs::*;
}

pub type AwarenessRef = Arc<RwLock<yrs::sync::Awareness>>;
