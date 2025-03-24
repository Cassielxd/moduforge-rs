use std::sync::Arc;

use moduforge_core::state::plugin::{Plugin, PluginSpec};
use moduforge_runtime::extension::Extension;

use crate::plugins::{P1Plugin, P1State, P2Plugin};

pub fn get_extension() -> Extension {
    let mut extension = Extension::default();
    let plugin1 = Plugin::new(PluginSpec {
        state: Some(Arc::new(P1State)),
        key: ("name".to_owned(), "测试插件一".to_owned()),
        tr: Some(Arc::new(P1Plugin {})),
    });
    extension.add_plugin(Arc::new(plugin1));
    extension.add_plugin(Arc::new(Plugin::new(PluginSpec {
        state: None,
        key: ("name1".to_owned(), "测试插件二".to_owned()),
        tr: Some(Arc::new(P2Plugin {})),
    })));
    extension
}
