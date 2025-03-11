use std::sync::Arc;

use moduforge_core::state::plugin::{Plugin, PluginSpec};
use moduforge_runtime::extension::Extension;

use crate::plugins::{P1Plugin, P1State, P2Plugin};

pub fn get_extension() -> Extension {
  let mut extension = Extension::default();
  extension.add_plugin(
    Plugin::new(PluginSpec {
      state: Some(Arc::new(P1State)),
      key: ("name".to_owned(), "sadasdasdsad".to_owned()),
      tr: Some(Arc::new(P1Plugin {})),
    })
    .into(),
  );
  extension.add_plugin(
    Plugin::new(PluginSpec {
      state: None,
      key: ("name1".to_owned(), "sadasdasdsad1".to_owned()),
      tr: Some(Arc::new(P2Plugin {})),
    })
    .into(),
  );
  extension
}
