use std::sync::Arc;

use moduforge_state::plugin::{Plugin, PluginSpec};
use moduforge_core::extension::Extension;

use crate::plugins::{P1Plugin, P1StateField, P2Plugin};

pub fn get_extension() -> Extension {
    let mut extension = Extension::default();
    let plugin1 = Plugin::new(PluginSpec {
        state_field: Some(Arc::new(P1StateField)),
        key: ("name".to_owned(), "测试插件一".to_owned()),
        tr: Some(Arc::new(P1Plugin {})),
        priority: 1,
    });
    extension.add_plugin(Arc::new(plugin1));
    extension.add_plugin(Arc::new(Plugin::new(PluginSpec {
        state_field: None,
        key: ("name1".to_owned(), "测试插件二".to_owned()),
        tr: Some(Arc::new(P2Plugin {})),
        priority: 2,
    })));
    extension.add_op_fn(Arc::new(move |op_state| {
        op_state.put(MyGlobalTest);
        Ok(())
    }));
    extension
}

#[derive(Clone, Default, Debug)]
pub struct MyGlobalTest;
impl MyGlobalTest {
    pub fn new() -> MyGlobalTest {
        MyGlobalTest
    }
    pub fn print(&self) {
        println!("全局 MyGlobalTest");
    }
}
