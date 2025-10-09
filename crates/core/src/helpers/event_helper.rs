//! 事件管理辅助模块
//!
//! 提供统一的事件管理逻辑，包括：
//! - 事件总线初始化
//! - 事件广播
//! - 事件处理器注册
//! - 事件循环管理

use crate::{
    config::ForgeConfig,
    debug::debug,
    error::{error_utils, ForgeResult},
    event::{Event, EventBus},
    metrics,
    types::RuntimeOptions,
};
use mf_state::state::State;
use std::sync::Arc;

/// 事件管理辅助器
pub struct EventHelper;

impl EventHelper {
    /// 创建并初始化事件总线
    ///
    /// # 参数
    /// * `config` - Forge配置
    /// * `runtime_options` - 运行时选项
    /// * `state` - 初始状态
    ///
    /// # 返回值
    /// * `ForgeResult<EventBus<Event>>` - 已初始化的事件总线或错误
    pub async fn create_and_init_event_bus(
        config: &ForgeConfig,
        runtime_options: &RuntimeOptions,
        state: Arc<State>,
    ) -> ForgeResult<EventBus<Event>> {
        let event_bus = EventBus::with_config(config.event.clone());
        debug!("已创建事件总线");

        // 注册事件处理器
        let handlers = runtime_options.get_event_handlers();
        event_bus.add_event_handlers(handlers)?;

        // 启动事件循环
        event_bus.start_event_loop();
        debug!("事件总线已启动");

        // 广播创建事件
        event_bus.broadcast_blocking(Event::Create(state)).map_err(|e| {
            error_utils::event_error(format!("广播 Create 事件失败: {e}"))
        })?;

        Ok(event_bus)
    }

    /// 广播事件（异步）
    ///
    /// # 参数
    /// * `event_bus` - 事件总线的可变引用
    /// * `event` - 要广播的事件
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub async fn emit_event(
        event_bus: &mut EventBus<Event>,
        event: Event,
    ) -> ForgeResult<()> {
        metrics::event_emitted(event.name());
        event_bus.broadcast(event).await?;
        Ok(())
    }

    /// 广播事件（同步阻塞）
    ///
    /// # 参数
    /// * `event_bus` - 事件总线的可变引用
    /// * `event` - 要广播的事件
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub fn emit_event_blocking(
        event_bus: &mut EventBus<Event>,
        event: Event,
    ) -> ForgeResult<()> {
        metrics::event_emitted(event.name());
        event_bus.broadcast_blocking(event)?;
        Ok(())
    }

    /// 销毁事件总线（异步）
    ///
    /// # 参数
    /// * `event_bus` - 事件总线的可变引用
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub async fn destroy_event_bus(
        event_bus: &mut EventBus<Event>
    ) -> ForgeResult<()> {
        // 先广播销毁事件
        event_bus.broadcast(Event::Destroy).await?;
        // 然后停止事件循环
        event_bus.destroy().await?;
        debug!("事件总线已销毁");
        Ok(())
    }

    /// 销毁事件总线（同步阻塞）
    ///
    /// # 参数
    /// * `event_bus` - 事件总线的可变引用
    pub fn destroy_event_bus_blocking(event_bus: &mut EventBus<Event>) {
        event_bus.destroy_blocking();
        debug!("事件总线已销毁（同步）");
    }
}
