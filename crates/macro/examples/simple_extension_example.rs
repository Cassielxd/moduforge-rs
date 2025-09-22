use mf_macro::{
    mf_extension, mf_extension_with_config, mf_op, mf_global_attr, mf_ops,
};
use mf_core::ForgeResult;
use mf_state::ops::GlobalResourceManager;

// ===== 简单的操作函数定义 =====

// 使用 mf_op! 宏定义操作函数
mf_op!(init_service, {
    println!("服务初始化完成");
    Ok(())
});

mf_op!(start_monitoring, |_manager| {
    println!("监控系统启动");
    Ok(())
});

mf_op!(cleanup_resources, {
    println!("资源清理完成");
    Ok(())
});

// 传统函数定义（用于 mf_ops! 宏）
fn backup_data(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("数据备份完成");
    Ok(())
}

fn restore_data(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("数据恢复完成");
    Ok(())
}

// ===== 操作块定义 =====

// 使用 mf_ops! 宏创建操作块
mf_ops!(maintenance_ops, [backup_data, restore_data]);

// ===== 扩展定义 =====

// 1. 基础服务扩展
mf_extension!(
    service_extension,
    ops = [init_service, start_monitoring],
    global_attributes = [
        mf_global_attr!("service", "name", "my_service"),
        mf_global_attr!("service", "port", "8080"),
        mf_global_attr!("service", "timeout", "30")
    ],
    docs = "基础服务扩展，包含服务初始化和监控"
);

// 2. 完整的服务扩展
mf_extension!(
    full_service_extension,
    ops = [init_service, start_monitoring, cleanup_resources],
    global_attributes = [
        mf_global_attr!("service", "name", "full_service"),
        mf_global_attr!("service", "environment", "production"),
        mf_global_attr!("monitoring", "enabled", "true"),
        mf_global_attr!("monitoring", "interval", "60")
    ],
    docs = "完整的服务扩展，包含初始化、监控和清理功能"
);

// 3. 可配置的服务扩展
mf_extension_with_config!(
    configurable_service_extension,
    config = {
        service_name: String,
        port: u16,
        enable_monitoring: bool
    },
    init_fn = |ext: &mut mf_core::extension::Extension,
               service_name: String,
               port: u16,
               enable_monitoring: bool| {
        // 添加基础配置
        ext.add_global_attribute(mf_global_attr!("service", "name", &service_name));
        ext.add_global_attribute(mf_global_attr!("service", "port", &port.to_string()));

        // 添加基础操作
        ext.add_op_fn(std::sync::Arc::new(init_service));

        // 根据配置添加监控功能
        if enable_monitoring {
            ext.add_global_attribute(mf_global_attr!("monitoring", "enabled", "true"));
            ext.add_op_fn(std::sync::Arc::new(start_monitoring));
        }

        // 总是添加清理功能
        ext.add_op_fn(std::sync::Arc::new(cleanup_resources));
    },
    docs = "可配置的服务扩展，支持动态启用监控功能"
);

// ===== 使用示例 =====

pub fn main() -> ForgeResult<()> {
    println!("=== ModuForge 扩展宏示例 ===\n");

    // 1. 基础服务扩展
    println!("1. 初始化基础服务扩展:");
    let basic_ext = service_extension::init();
    println!("   - 操作函数数量: {}", basic_ext.get_op_fns().len());
    println!("   - 全局属性数量: {}", basic_ext.get_global_attributes().len());

    // 执行操作函数
    let manager = GlobalResourceManager::default();
    println!("   执行操作函数:");
    for (i, op_fn) in basic_ext.get_op_fns().iter().enumerate() {
        match op_fn(&manager) {
            Ok(()) => println!("     操作 {} 执行成功", i + 1),
            Err(e) => println!("     操作 {} 执行失败: {:?}", i + 1, e),
        }
    }

    println!();

    // 2. 完整服务扩展
    println!("2. 初始化完整服务扩展:");
    let full_ext = full_service_extension::init();
    println!("   - 操作函数数量: {}", full_ext.get_op_fns().len());
    println!("   - 全局属性数量: {}", full_ext.get_global_attributes().len());

    // 显示全局属性
    println!("   全局属性:");
    for attr in full_ext.get_global_attributes() {
        for key in attr.keys() {
            println!("     - {}: 已配置", key);
        }
    }

    println!();

    // 3. 可配置服务扩展
    println!("3. 初始化可配置服务扩展:");
    let config_ext_with_monitoring = configurable_service_extension::init(
        "webapp".to_string(),
        3000,
        true, // 启用监控
    );
    println!("   带监控配置:");
    println!(
        "   - 操作函数数量: {}",
        config_ext_with_monitoring.get_op_fns().len()
    );
    println!(
        "   - 全局属性数量: {}",
        config_ext_with_monitoring.get_global_attributes().len()
    );

    let config_ext_no_monitoring = configurable_service_extension::init(
        "api".to_string(),
        8080,
        false, // 不启用监控
    );
    println!("   不带监控配置:");
    println!(
        "   - 操作函数数量: {}",
        config_ext_no_monitoring.get_op_fns().len()
    );
    println!(
        "   - 全局属性数量: {}",
        config_ext_no_monitoring.get_global_attributes().len()
    );

    println!();

    // 4. 使用操作块
    println!("4. 测试维护操作块:");
    let maintenance = maintenance_ops();
    println!("   - 维护操作数量: {}", maintenance.len());

    for (i, op_fn) in maintenance.iter().enumerate() {
        match op_fn(&manager) {
            Ok(()) => println!("     维护操作 {} 执行成功", i + 1),
            Err(e) => println!("     维护操作 {} 执行失败: {:?}", i + 1, e),
        }
    }

    println!();

    // 5. 演示宏的类型安全性
    println!("5. 类型安全演示:");
    println!("   所有扩展都是类型安全的，编译时验证正确性");
    println!(
        "   - 操作函数签名统一: fn(&GlobalResourceManager) -> ForgeResult<()>"
    );
    println!("   - 全局属性自动类型转换");
    println!("   - 配置参数类型检查");

    println!("\n=== 示例完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_extension() {
        let ext = service_extension::init();
        assert_eq!(ext.get_op_fns().len(), 2);
        assert_eq!(ext.get_plugins().len(), 0);
        assert_eq!(ext.get_global_attributes().len(), 3);
    }

    #[test]
    fn test_full_service_extension() {
        let ext = full_service_extension::init();
        assert_eq!(ext.get_op_fns().len(), 3);
        assert_eq!(ext.get_plugins().len(), 0);
        assert_eq!(ext.get_global_attributes().len(), 4);
    }

    #[test]
    fn test_configurable_service_extension() {
        // 测试启用监控的配置
        let ext_with_monitoring = configurable_service_extension::init(
            "test_service".to_string(),
            8080,
            true,
        );
        assert_eq!(ext_with_monitoring.get_op_fns().len(), 3); // init, start_monitoring, cleanup
        assert!(ext_with_monitoring.get_global_attributes().len() >= 3);

        // 测试不启用监控的配置
        let ext_no_monitoring = configurable_service_extension::init(
            "test_service".to_string(),
            8080,
            false,
        );
        assert_eq!(ext_no_monitoring.get_op_fns().len(), 2); // init, cleanup (没有 start_monitoring)
        assert!(ext_no_monitoring.get_global_attributes().len() >= 2);
    }

    #[test]
    fn test_maintenance_ops() {
        let ops = maintenance_ops();
        assert_eq!(ops.len(), 2);

        let manager = GlobalResourceManager::default();
        for op_fn in ops {
            assert!(op_fn(&manager).is_ok());
        }
    }

    #[test]
    fn test_operation_functions() {
        let manager = GlobalResourceManager::default();

        // 测试各个操作函数
        assert!(init_service(&manager).is_ok());
        assert!(start_monitoring(&manager).is_ok());
        assert!(cleanup_resources(&manager).is_ok());
        assert!(backup_data(&manager).is_ok());
        assert!(restore_data(&manager).is_ok());
    }

    #[test]
    fn test_global_attributes_structure() {
        let ext = service_extension::init();
        let attrs = ext.get_global_attributes();

        // 验证属性结构
        assert!(!attrs.is_empty());

        // 检查每个属性都有正确的键
        for attr in attrs {
            assert!(!attr.keys().is_empty(), "每个全局属性都应该有键");
        }
    }
}
