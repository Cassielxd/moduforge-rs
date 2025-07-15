use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use yrs::TransactionMut;
use mf_transform::step::Step;
use crate::types::StepResult;
use super::error::{ConversionError, ConversionResult};
use super::typed_converter::{ErasedConverter, ConversionContext, ConverterInfo};

/// 高性能的静态分发转换器注册表
/// 使用编译时注册和运行时 O(1) 查找
pub struct StaticConverterRegistry {
    /// 类型ID到转换器的映射 - 主要查找路径
    converters: HashMap<TypeId, Arc<ErasedConverter>>,
    /// 按优先级排序的转换器列表 - 用于fallback
    ordered_converters: Vec<Arc<ErasedConverter>>,
    /// 转换器信息缓存
    converter_info: HashMap<TypeId, ConverterInfo>,
    /// 性能统计
    performance_stats: PerformanceStats,
}

impl StaticConverterRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            converters: HashMap::new(),
            ordered_converters: Vec::new(),
            converter_info: HashMap::new(),
            performance_stats: PerformanceStats::new(),
        }
    }

    /// 注册转换器（编译时调用）
    pub fn register_converter<T, C>(&mut self) -> &mut Self
    where
        T: Step + 'static,
        C: super::typed_converter::TypedStepConverter<T> + Default + 'static,
    {
        let type_id = TypeId::of::<T>();
        let converter = Arc::new(ErasedConverter::new::<T, C>());
        
        // 检查重复注册
        if self.converters.contains_key(&type_id) {
            tracing::warn!(
                "转换器重复注册: {} for type {}",
                converter.converter_name(),
                converter.type_name()
            );
            return self;
        }

        // 存储转换器信息
        let info = ConverterInfo {
            type_name: converter.type_name(),
            converter_name: converter.converter_name(),
            priority: converter.priority(),
            supports_concurrent: converter.supports_concurrent(),
            step_type_id: type_id,
        };

        self.converter_info.insert(type_id, info);
        self.converters.insert(type_id, converter.clone());
        
        // 按优先级插入有序列表
        let insert_pos = self.ordered_converters
            .iter()
            .position(|c| c.priority() > converter.priority())
            .unwrap_or(self.ordered_converters.len());
        
        self.ordered_converters.insert(insert_pos, converter);

        tracing::info!(
            "✅ 转换器已注册: {} for {} (优先级: {})",
            C::converter_name(),
            std::any::type_name::<T>(),
            C::priority()
        );

        self
    }

    /// 查找并应用转换器 - 主要性能路径
    pub fn convert_step(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        let start_time = std::time::Instant::now();
        let step_type_id = step.type_id();

        // O(1) 精确匹配查找
        if let Some(converter) = self.converters.get(&step_type_id) {
            let result = converter.try_convert(step, txn, context);
            
            // 更新性能统计
            self.performance_stats.record_conversion(
                step_type_id,
                start_time.elapsed(),
                result.is_ok(),
            );
            
            return result;
        }

        // 如果没有找到精确匹配，尝试有序列表（用于兼容性）
        for converter in &self.ordered_converters {
            if converter.type_id() == step_type_id {
                let result = converter.try_convert(step, txn, context);
                self.performance_stats.record_conversion(
                    step_type_id,
                    start_time.elapsed(),
                    result.is_ok(),
                );
                return result;
            }
        }

        // 没有找到任何匹配的转换器
        self.performance_stats.record_conversion(
            step_type_id,
            start_time.elapsed(),
            false,
        );

        Err(ConversionError::UnsupportedStepType {
            step_type: step.name().to_string(),
            type_id: step_type_id,
        })
    }

    /// 批量转换步骤 - 优化的批处理路径
    pub fn convert_steps_batch(
        &self,
        steps: &[&dyn Step],
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> Vec<ConversionResult<StepResult>> {
        let mut results = Vec::with_capacity(steps.len());
        
        // 按类型对步骤进行分组以提高缓存效率
        let mut grouped_steps: HashMap<TypeId, Vec<&dyn Step>> = HashMap::new();
        for step in steps {
            grouped_steps
                .entry(step.type_id())
                .or_insert_with(Vec::new)
                .push(*step);
        }

        // 按组处理步骤
        for (type_id, group_steps) in grouped_steps {
            if let Some(converter) = self.converters.get(&type_id) {
                for step in group_steps {
                    let result = converter.try_convert(step, txn, context);
                    results.push(result);
                }
            } else {
                // 为未找到转换器的步骤添加错误
                for step in group_steps {
                    results.push(Err(ConversionError::UnsupportedStepType {
                        step_type: step.name().to_string(),
                        type_id,
                    }));
                }
            }
        }

        results
    }

    /// 验证步骤而不执行转换
    pub fn validate_step(
        &self,
        step: &dyn Step,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        let step_type_id = step.type_id();
        
        if let Some(_converter) = self.converters.get(&step_type_id) {
            // TODO: 实现验证逻辑
            Ok(())
        } else {
            Err(ConversionError::UnsupportedStepType {
                step_type: step.name().to_string(),
                type_id: step_type_id,
            })
        }
    }

    /// 检查是否支持某个步骤类型
    pub fn supports_step_type(&self, type_id: TypeId) -> bool {
        self.converters.contains_key(&type_id)
    }

    /// 获取所有已注册的转换器信息
    pub fn get_all_converter_info(&self) -> Vec<&ConverterInfo> {
        self.converter_info.values().collect()
    }

    /// 获取特定类型的转换器信息
    pub fn get_converter_info(&self, type_id: TypeId) -> Option<&ConverterInfo> {
        self.converter_info.get(&type_id)
    }

    /// 获取性能统计信息
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    /// 清空所有转换器（主要用于测试）
    pub fn clear(&mut self) {
        self.converters.clear();
        self.ordered_converters.clear();
        self.converter_info.clear();
        self.performance_stats = PerformanceStats::new();
    }

    /// 获取注册的转换器数量
    pub fn converter_count(&self) -> usize {
        self.converters.len()
    }
}

impl Default for StaticConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能统计信息
#[derive(Debug)]
pub struct PerformanceStats {
    /// 总转换次数
    total_conversions: std::sync::atomic::AtomicU64,
    /// 成功转换次数
    successful_conversions: std::sync::atomic::AtomicU64,
    /// 按类型的转换统计
    type_stats: RwLock<HashMap<TypeId, TypeConversionStats>>,
    /// 创建时间
    created_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct TypeConversionStats {
    pub total_count: u64,
    pub success_count: u64,
    pub total_duration: std::time::Duration,
    pub avg_duration: std::time::Duration,
    pub min_duration: std::time::Duration,
    pub max_duration: std::time::Duration,
}

impl PerformanceStats {
    pub fn new() -> Self {
        Self {
            total_conversions: std::sync::atomic::AtomicU64::new(0),
            successful_conversions: std::sync::atomic::AtomicU64::new(0),
            type_stats: RwLock::new(HashMap::new()),
            created_at: std::time::Instant::now(),
        }
    }

    pub fn record_conversion(
        &self,
        type_id: TypeId,
        duration: std::time::Duration,
        success: bool,
    ) {
        use std::sync::atomic::Ordering;

        // 更新全局统计
        self.total_conversions.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_conversions.fetch_add(1, Ordering::Relaxed);
        }

        // 更新类型特定统计
        let mut type_stats = self.type_stats.write().unwrap();
        let stats = type_stats.entry(type_id).or_insert_with(|| TypeConversionStats {
            total_count: 0,
            success_count: 0,
            total_duration: std::time::Duration::ZERO,
            avg_duration: std::time::Duration::ZERO,
            min_duration: std::time::Duration::MAX,
            max_duration: std::time::Duration::ZERO,
        });

        stats.total_count += 1;
        if success {
            stats.success_count += 1;
        }
        
        stats.total_duration += duration;
        stats.avg_duration = stats.total_duration / stats.total_count as u32;
        stats.min_duration = stats.min_duration.min(duration);
        stats.max_duration = stats.max_duration.max(duration);
    }

    pub fn get_total_conversions(&self) -> u64 {
        self.total_conversions.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_success_rate(&self) -> f64 {
        let total = self.get_total_conversions();
        if total == 0 {
            0.0
        } else {
            let successful = self.successful_conversions.load(std::sync::atomic::Ordering::Relaxed);
            successful as f64 / total as f64
        }
    }

    pub fn get_type_stats(&self, type_id: TypeId) -> Option<TypeConversionStats> {
        self.type_stats.read().unwrap().get(&type_id).cloned()
    }

    pub fn get_uptime(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

/// 全局注册表单例
static GLOBAL_REGISTRY: OnceLock<RwLock<StaticConverterRegistry>> = OnceLock::new();

/// 获取全局转换器注册表
pub fn global_registry() -> &'static RwLock<StaticConverterRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| RwLock::new(StaticConverterRegistry::new()))
}

/// 注册转换器到全局注册表的便捷函数
pub fn register_global_converter<T, C>()
where
    T: Step + 'static,
    C: super::typed_converter::TypedStepConverter<T> + Default + 'static,
{
    let mut registry = global_registry().write().unwrap();
    registry.register_converter::<T, C>();
}

/// 使用全局注册表转换步骤的便捷函数
pub fn convert_step_global(
    step: &dyn Step,
    txn: &mut TransactionMut,
    context: &ConversionContext,
) -> ConversionResult<StepResult> {
    let registry = global_registry().read().unwrap();
    registry.convert_step(step, txn, context)
}

/// 获取全局注册表的性能统计
pub fn get_global_performance_stats() -> std::sync::RwLockReadGuard<'static, StaticConverterRegistry> {
    global_registry().read().unwrap()
}
