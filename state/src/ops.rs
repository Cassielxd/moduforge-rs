use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::{
    gotham_state::GothamState,
    resource_table::{ResourceId, ResourceTable},
};

#[derive(Default, Debug)]
pub struct OpState {
    pub resource_table: ResourceTable,
    pub gotham_state: GothamState,
    pub unrefed_resources: HashSet<ResourceId>,
}
unsafe impl Send for OpState {}
unsafe impl Sync for OpState {}
impl OpState {
    pub fn new() -> Self {
        Self {
            resource_table: ResourceTable::default(),
            gotham_state: GothamState::default(),
            unrefed_resources: Default::default(),
        }
    }
    pub(crate) fn clear(&mut self) {
        std::mem::take(&mut self.gotham_state);
        std::mem::take(&mut self.resource_table);
    }

    pub fn has_ref(
        &self,
        resource_id: ResourceId,
    ) -> bool {
        !self.unrefed_resources.contains(&resource_id)
    }
}

impl Deref for OpState {
    type Target = GothamState;

    fn deref(&self) -> &Self::Target {
        &self.gotham_state
    }
}

impl DerefMut for OpState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gotham_state
    }
}
