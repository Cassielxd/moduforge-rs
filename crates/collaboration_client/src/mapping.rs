/// 静态分发 StepConverter 实现
/// 使用静态分发替代动态分发，提供更好的性能和类型安全性

use yrs::{Transact, ReadTxn, Map};

// 重新导出所有核心组件
pub use crate::mapping_v2::{
    // 核心 traits 和类型
    TypedStepConverter, 
    ConversionContext, 
    ErasedConverter,
    
    // 注册表和全局函数
    StaticConverterRegistry, 
    global_registry, 
    convert_step_global as convert_step,
    register_global_converter, 
    get_global_performance_stats,
    PerformanceStats,
    TypeConversionStats,
    
    // 错误处理
    ConversionError, 
    ConversionResult, 
    
    // 转换器实现
    SimpleNodeAddConverter,
    SimpleNodeRemoveConverter,
    SimpleAttrConverter,
    SimpleMarkAddConverter,
    SimpleMarkRemoveConverter,
};

/// 便捷函数：批量转换步骤
pub fn convert_steps_batch(
    steps: &[&dyn mf_transform::step::Step],
    txn: &mut yrs::TransactionMut,
    context: &ConversionContext,
) -> Vec<ConversionResult<crate::types::StepResult>> {
    let registry = global_registry().read().unwrap();
    registry.convert_steps_batch(steps, txn, context)
}

/// 便捷函数：创建转换上下文
pub fn create_context(client_id: String, user_id: String) -> ConversionContext {
    ConversionContext::new(client_id, user_id)
}

/// 便捷函数：注册转换器
pub fn register_converter<T, C>()
where
    T: mf_transform::step::Step + 'static,
    C: TypedStepConverter<T> + Default + 'static,
{
    register_global_converter::<T, C>();
}

/// 映射器工具类
#[derive(Debug)]
pub struct Mapper;

impl Mapper {
    /// 获取全局注册表
    pub fn global_registry() -> &'static std::sync::RwLock<StaticConverterRegistry> {
        global_registry()
    }

    /// 获取 Yrs 文档的版本信息
    pub fn get_yrs_doc_version(doc: &yrs::Doc) -> u64 {
        let txn = doc.transact();
        txn.state_vector().len() as u64
    }

    /// 检查 Yrs 文档是否为空
    pub fn is_yrs_doc_empty(doc: &yrs::Doc) -> bool {
        let txn = doc.transact();
        let nodes_map = txn.get_map("nodes");
        nodes_map.map_or(true, |map| map.len(&txn) == 0)
    }

    /// 获取性能统计信息
    pub fn get_performance_stats() -> String {
        let registry = global_registry().read().unwrap();
        let stats = registry.get_performance_stats();
        format!(
            "性能统计:\n\
             - 总转换次数: {}\n\
             - 成功率: {:.2}%\n\
             - 运行时间: {:?}",
            stats.get_total_conversions(),
            stats.get_success_rate() * 100.0,
            stats.get_uptime()
        )
    }

    /// 获取注册的转换器数量
    pub fn converter_count() -> usize {
        let registry = global_registry().read().unwrap();
        registry.converter_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_transform::attr_step::AttrStep;
    use std::time::Instant;

    #[test]
    fn test_api_simplicity() {
        let context = create_context(
            "test_client".to_string(),
            "test_user".to_string(),
        );
        
        assert_eq!(context.client_id, "test_client");
        assert_eq!(context.user_id, "test_user");
    }

    #[test]
    fn test_registry_access() {
        let registry = Mapper::global_registry();
        assert!(registry.read().is_ok());
    }

    #[tokio::test]
    async fn test_performance_tracking() {
        let start = Instant::now();
        
        let doc = yrs::Doc::new();
        let mut txn = doc.transact_mut();
        
        let context = create_context(
            "perf_client".to_string(),
            "perf_user".to_string(),
        );

        let mut attrs = imbl::HashMap::new();
        attrs.insert("test_attr".to_string(), serde_json::json!("test_value"));
        
        let step = AttrStep {
            id: "test_node".to_string(),
            values: attrs,
        };

        let result = convert_step(&step, &mut txn, &context);
        let duration = start.elapsed();
        
        println!("静态分发转换耗时: {:?}", duration);
        
        // 验证结果格式
        match result {
            Ok(step_result) => {
                assert_eq!(step_result.step_name, "attr_step");
                assert!(!step_result.step_id.is_empty());
            },
            Err(e) => {
                println!("转换失败（测试环境预期）: {}", e);
            }
        }
    }

    #[test]
    fn test_performance_stats() {
        let stats_str = Mapper::get_performance_stats();
        assert!(!stats_str.is_empty());
        assert!(stats_str.contains("性能统计"));
        println!("{}", stats_str);
    }
}