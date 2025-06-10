//! # 自定义函数监听器模块
//!
//! 本模块实现了CustomListener，用于在JavaScript运行时环境中注册和管理自定义函数。
//! 
//! ## 主要功能
//! 
//! - **函数注册**: 在运行时启动时自动将Rust自定义函数注册到JavaScript全局作用域
//! - **类型转换**: 处理Rust和JavaScript之间的数据类型转换
//! - **异步支持**: 提供异步函数调用支持，确保不阻塞JavaScript执行
//! - **错误处理**: 完善的错误捕获和处理机制
//! 
//! ## 使用场景
//! 
//! 该监听器主要用于规则引擎中，允许在规则表达式中调用预定义的Rust函数，
//! 从而扩展JavaScript运行时的功能。
//! 
//! ## 架构说明
//! 
//! ```text
//! CustomFunctionRegistry → CustomListener → JavaScript Runtime
//!        ↓                      ↓                    ↓
//!    函数定义存储           函数注册处理          函数调用执行
//! ```

use std::future::Future;
use std::pin::Pin;


use crate::handler::function::error::{FunctionResult, ResultExt};
use crate::handler::function::listener::{RuntimeEvent, RuntimeListener};
use crate::handler::function::serde::JsValue;
use moduforge_rules_expression::{CustomFunctionRegistry, functions::arguments::Arguments};
use rquickjs::prelude::{Async, Func};
use rquickjs::{CatchResultExt, Ctx};

/// 自定义函数监听器
/// 
/// 该监听器负责在JavaScript运行时启动时，将所有注册的自定义函数
/// 绑定到JavaScript全局作用域中，使得这些函数可以在规则表达式中被调用
/// 
/// # 工作流程
/// 1. 监听运行时启动事件
/// 2. 从CustomFunctionRegistry获取所有已注册的函数
/// 3. 将每个函数包装为异步JavaScript函数
/// 4. 注册到JavaScript全局变量中
pub struct CustomListener {
    // 目前为空结构体，后续可以添加配置或状态字段
}

impl RuntimeListener for CustomListener {
    /// 处理运行时事件的核心方法
    /// 
    /// # 参数
    /// - `ctx`: QuickJS上下文，用于操作JavaScript环境
    /// - `event`: 运行时事件类型
    /// 
    /// # 返回值
    /// 返回一个异步Future，包含操作结果
    fn on_event<'js>(
        &self,
        ctx: Ctx<'js>,
        event: RuntimeEvent,
    ) -> Pin<Box<dyn Future<Output = FunctionResult> + 'js>> {
        Box::pin(async move {
            // 只在运行时启动事件时执行函数注册
            if event != RuntimeEvent::Startup {
                return Ok(());
            };
            
            // 设置全局函数及变量
            // 从自定义函数注册表中获取所有函数名称
            let functions_keys = CustomFunctionRegistry::list_functions();
           
            // 遍历每个注册的函数
            for function_key in functions_keys {
                // 根据函数名获取函数定义
                let function_definition = CustomFunctionRegistry::get_definition(&function_key);
                
                if let Some(function_definition) = function_definition {
                    // 将Rust函数包装为JavaScript异步函数并注册到全局作用域
                    ctx.globals()
                        .set(
                            function_key, // 函数名作为全局变量名
                            Func::from(Async(
                                move |ctx: Ctx<'js>, context: JsValue| {
                                    // 克隆函数定义以避免生命周期问题
                                    let function_definition = function_definition.clone();
                                    
                                    async move {
                                        // 调用Rust函数，传入JavaScript参数
                                        let response = function_definition
                                            .call(Arguments(&[context.0]))
                                            .or_throw(&ctx)?;
                                        
                                        // 将Rust函数的返回值序列化为JSON，再转换为JavaScript值
                                        let k = serde_json::to_value(response)
                                            .or_throw(&ctx)?
                                            .into();
                                        
                                        return rquickjs::Result::Ok(JsValue(k));
                                    }
                                },
                            )),
                        )
                        .catch(&ctx)?; // 捕获并处理可能的JavaScript异常
                }
            }

            Ok(()) // 成功完成函数注册
        })
    }
}
