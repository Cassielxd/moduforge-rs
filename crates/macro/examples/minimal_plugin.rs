use mf_macro::{mf_plugin, mf_plugin_metadata};

/// 最简单的插件示例 - 所有功能都是可选的
/// 只需要提供插件名称，其他都可以省略
mf_plugin!(minimal_plugin, docs = "最简单的插件 - 展示所有功能都是可选的");

fn main() {
    println!("=== 最简插件示例 ===\n");

    // 创建最简插件
    let plugin = minimal_plugin::new();
    let metadata = plugin.get_metadata();

    println!("📦 插件信息:");
    println!("   - 名称: {}", metadata.name);
    println!("   - 版本: {}", metadata.version);
    println!("   - 描述: {}", metadata.description);
    println!("   - 作者: {}", metadata.author);
    println!();

    // 检查插件规范
    let spec = minimal_plugin::spec();
    println!("🔧 插件规范:");
    println!("   - 有状态字段: {}", spec.state_field.is_some());
    println!("   - 插件名称: {}", spec.tr.metadata().name);
    println!();

    println!("✅ 最简插件创建成功！");
    println!("💡 说明: 此插件不提供任何自定义功能");
    println!("   - 没有 metadata 配置（使用默认）");
    println!("   - 没有 config 配置（不实现 config 方法）");
    println!("   - 没有 append_transaction（不实现处理方法）");
    println!("   - 没有 filter_transaction（不实现过滤方法）");
    println!("   - 没有 state_field（无状态管理）");

    println!("\n=== 这就是真正的可选性！ ===");
}
