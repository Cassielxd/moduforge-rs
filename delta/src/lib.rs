pub mod delta;
pub mod diff;

use std::sync::Arc;

use moduforge_core::model::node_pool::{NodePool, NodePoolInner};






pub fn to_binary(node_pool:&NodePool) -> Result<Vec<u8>, bincode::error::EncodeError> {
    let config = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_no_limit();
    
    bincode::encode_to_vec(&*node_pool.inner, config)
}

pub fn from_binary(bytes: &[u8]) -> Result<NodePool, bincode::error::DecodeError> {
    let config = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_no_limit();
    
    let inner: NodePoolInner = bincode::decode_from_slice(bytes, config)?.0;
    Ok(NodePool {
        inner: Arc::new(inner),
    })
}