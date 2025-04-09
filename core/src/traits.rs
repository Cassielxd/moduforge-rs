use std::sync::Arc;
use crate::{
    event::EventBus, extension_manager::ExtensionManager,
    history_manager::HistoryManager, types::EditorOptions,
};
use async_trait::async_trait;

use moduforge_state::{
    state::State,
    transaction::{Command, Transaction},
};
use moduforge_model::{node_pool::NodePool, schema::Schema};

