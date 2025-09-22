use std::{cell::RefCell, rc::Rc};

use deno_core::{op2};

use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;

use crate::worker::ChannelRequest;

pub struct TransactionResource(pub mf_state::Transaction);
impl Resource for TransactionResource{}

#[op2(fast)]
#[smi]
pub  fn op_new_transaction(
   state: &mut OpState,
  #[smi] rid: ResourceId,
)->Result<ResourceId,  deno_core::error::ResourceError>{
    let request = state.resource_table.get::<ChannelRequest>(rid)?;
    let tr = request.trs.last().unwrap().new_shared();
    let id = state.resource_table.add(TransactionResource(tr));
    Ok(id)
}

#[op2(fast)]
#[smi]
pub  fn op_response_transaction(
   state: &mut OpState,
  #[smi] rid: ResourceId,
)->Result<ResourceId,  deno_core::error::ResourceError>{
    let request = state.resource_table.get::<ChannelRequest>(rid)?;
    let tr = request.trs.last().unwrap().new_shared();
    let id = state.resource_table.add(TransactionResource(tr));
    Ok(id)
}