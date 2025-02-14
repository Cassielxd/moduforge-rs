use std::env::current_dir;

use zen_engine::{
    handler::custom_node_adapter::NoopCustomNode,
    loader::{FilesystemLoader, FilesystemLoaderOptions},
    DecisionEngine,
};

//获取默认的引擎
pub fn get_engine() -> DecisionEngine<FilesystemLoader, NoopCustomNode> {
    let engine: DecisionEngine<FilesystemLoader, NoopCustomNode> =
        DecisionEngine::default().with_loader(create_fs_loader().into());
    engine
}
/// 创建文件系统加载器
pub fn create_fs_loader() -> FilesystemLoader {
    FilesystemLoader::new(FilesystemLoaderOptions {
        keep_in_memory: false,
        root: rules_data_root(),
    })
}
/// 获取规则数据根目录

pub fn rules_data_root() -> String {
    let p = current_dir().unwrap().join("rules");
    let cargo_root = p.as_path();
    cargo_root.to_string_lossy().to_string()
}
