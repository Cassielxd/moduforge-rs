//! 演示快照生成的构建脚本

use mf_core::build_tools::{generate_snapshot_with_config, BuildSnapshotConfig};
use mf_core::config::Environment;

fn main() {
    // 配置快照生成参数
    let config = BuildSnapshotConfig::default()
        .with_environment(Environment::Production)
        .with_snapshot_name("demo_snapshot.bin")
        .with_schema_dir("schemas")
        .with_config_override("processor.max_queue_size", "10000")
        .with_config_override("performance.enable_monitoring", "true");

    // 生成快照
    if let Err(e) = generate_snapshot_with_config(config) {
        panic!("生成快照失败: {}", e);
    }

    println!("cargo:warning=快照生成完成");
}