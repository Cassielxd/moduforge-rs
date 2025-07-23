use dashmap::DashMap;

use crate::{types::EditorTrait, ContextHelper};

pub mod editor;

pub async fn init_contex() {
    let map_p: DashMap<String, Box<dyn EditorTrait>> = DashMap::new();
    ContextHelper::set(map_p);
}
