use std::any::{Any, TypeId};
use yrs::TransactionMut;
use mf_transform::step::Step;
use crate::types::StepResult;
use super::error::{ConversionError, ConversionResult};

/// 类型安全的步骤转换器 trait
/// 使用泛型参数确保编译时类型安全
pub trait TypedStepConverter<T>: Send + Sync + 'static
where
    T: Step + 'static,
{
    /// 转换具体类型的步骤到 Yrs 事务
    fn convert_typed(
        &self,
        step: &T,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult>;

    /// 验证步骤是否有效（可选实现）
    fn validate_step(
        &self,
        _step: &T,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        // 默认实现：总是有效
        Ok(())
    }

    /// 获取转换器名称
    fn converter_name() -> &'static str
    where
        Self: Sized;

    /// 获取支持的步骤类型名称
    fn step_type_name() -> &'static str
    where
        Self: Sized;

    /// 转换器优先级（数字越小优先级越高）
    fn priority() -> u8
    where
        Self: Sized,
    {
        100
    }

    /// 是否支持并发执行
    fn supports_concurrent_execution() -> bool
    where
        Self: Sized,
    {
        true
    }
}

/// 转换上下文 - 提供转换过程中需要的信息
#[derive(Debug, Clone)]
pub struct ConversionContext {
    /// 客户端 ID
    pub client_id: String,
    /// 用户 ID  
    pub user_id: String,
    /// 时间戳
    pub timestamp: u64,
}

impl ConversionContext {
    pub fn new(
        client_id: String,
        user_id: String,
    ) -> Self {
        Self {
            client_id,
            user_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// 类型擦除的转换器包装器
/// 用于在运行时存储不同类型的转换器
pub struct ErasedConverter {
    type_id: TypeId,
    type_name: &'static str,
    converter_name: &'static str,
    priority: u8,
    supports_concurrent: bool,
    convert_fn: fn(
        &dyn Any,
        &mut TransactionMut,
        &ConversionContext,
    ) -> ConversionResult<StepResult>,
    validate_fn: fn(&dyn Any, &ConversionContext) -> ConversionResult<()>,
}

impl ErasedConverter {
    /// 创建类型擦除的转换器
    pub fn new<T, C>() -> Self
    where
        T: Step + 'static,
        C: TypedStepConverter<T> + Default + 'static,
    {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            converter_name: C::converter_name(),
            priority: C::priority(),
            supports_concurrent: C::supports_concurrent_execution(),
            convert_fn: |step_any, txn, context| {
                let converter = C::default();
                let step = step_any.downcast_ref::<T>().ok_or_else(|| {
                    ConversionError::unsupported_step::<T>("Type mismatch")
                })?;
                converter.convert_typed(step, txn, context)
            },
            validate_fn: |step_any, context| {
                let converter = C::default();
                let step = step_any.downcast_ref::<T>().ok_or_else(|| {
                    ConversionError::unsupported_step::<T>("Type mismatch")
                })?;
                converter.validate_step(step, context)
            },
        }
    }

    /// 尝试转换步骤
    pub fn try_convert(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        // 检查类型匹配
        if step.type_id() != self.type_id {
            return Err(ConversionError::UnsupportedStepType {
                step_type: step.name().to_string(),
                type_id: step.type_id(),
            });
        }

        // 先验证
        (self.validate_fn)(step as &dyn Any, context)?;

        // 然后转换
        (self.convert_fn)(step as &dyn Any, txn, context)
    }

    /// 获取类型信息
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn converter_name(&self) -> &'static str {
        self.converter_name
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }

    pub fn supports_concurrent(&self) -> bool {
        self.supports_concurrent
    }
}

impl std::fmt::Debug for ErasedConverter {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("ErasedConverter")
            .field("type_id", &self.type_id)
            .field("type_name", &self.type_name)
            .field("converter_name", &self.converter_name)
            .field("priority", &self.priority)
            .field("supports_concurrent", &self.supports_concurrent)
            .finish()
    }
}

/// 转换器工厂 trait - 用于延迟创建转换器实例
pub trait ConverterFactory: Send + Sync + 'static {
    fn create_converter(&self) -> Box<dyn Any + Send + Sync>;
    fn type_id(&self) -> TypeId;
    fn converter_info(&self) -> ConverterInfo;
}

/// 转换器信息
#[derive(Debug, Clone)]
pub struct ConverterInfo {
    pub type_name: &'static str,
    pub converter_name: &'static str,
    pub priority: u8,
    pub supports_concurrent: bool,
    pub step_type_id: TypeId,
}

/// 具体类型的转换器工厂实现
pub struct TypedConverterFactory<T, C>
where
    T: Step + 'static,
    C: TypedStepConverter<T> + Default + 'static,
{
    _phantom_step: std::marker::PhantomData<T>,
    _phantom_converter: std::marker::PhantomData<C>,
}

impl<T, C> TypedConverterFactory<T, C>
where
    T: Step + 'static,
    C: TypedStepConverter<T> + Default + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom_step: std::marker::PhantomData,
            _phantom_converter: std::marker::PhantomData,
        }
    }
}

impl<T, C> ConverterFactory for TypedConverterFactory<T, C>
where
    T: Step + 'static,
    C: TypedStepConverter<T> + Default + 'static,
{
    fn create_converter(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(C::default())
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn converter_info(&self) -> ConverterInfo {
        ConverterInfo {
            type_name: std::any::type_name::<T>(),
            converter_name: C::converter_name(),
            priority: C::priority(),
            supports_concurrent: C::supports_concurrent_execution(),
            step_type_id: TypeId::of::<T>(),
        }
    }
}

impl<T, C> Default for TypedConverterFactory<T, C>
where
    T: Step + 'static,
    C: TypedStepConverter<T> + Default + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
