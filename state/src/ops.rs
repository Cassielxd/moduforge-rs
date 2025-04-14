use std::{
    ops::{Deref, DerefMut},
};

use crate::{
    gotham_state::GothamState,
    resource_table::{ResourceTable},
};

#[derive(Default, Debug)]
pub struct GlobalResourceManager {
    pub resource_table: ResourceTable,
    pub gotham_state: GothamState
}
unsafe impl Send for GlobalResourceManager {}
unsafe impl Sync for GlobalResourceManager {}
impl GlobalResourceManager {
    pub fn new() -> Self {
        Self {
            resource_table: ResourceTable::default(),
            gotham_state: GothamState::default()
        }
    }
    pub(crate) fn clear(&mut self) {
        std::mem::take(&mut self.gotham_state);
        std::mem::take(&mut self.resource_table);
    }

}

impl Deref for GlobalResourceManager {
    type Target = GothamState;

    fn deref(&self) -> &Self::Target {
        &self.gotham_state
    }
}

impl DerefMut for GlobalResourceManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gotham_state
    }
}
