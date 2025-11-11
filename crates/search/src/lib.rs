pub mod backend;
pub mod backend_sqlite;
pub mod indexer;
pub mod model;
pub mod service;
pub mod state_plugin;
pub mod step_registry;

// 导出类型
pub use backend::{Backend, IndexMutation, SearchQuery, SqliteBackend};
pub use service::{
    IndexService, SearchService, IndexEvent, RebuildScope,
    event_from_transaction,
};
pub use state_plugin::{
    create_search_index_plugin, create_temp_search_index_plugin,
};
