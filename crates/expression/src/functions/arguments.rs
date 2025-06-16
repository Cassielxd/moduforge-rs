//! 函数参数处理模块
//!
//! 提供便捷的参数访问方法，支持类型检查和错误处理

use crate::variable::{DynamicVariable, RcCell};
use crate::Variable;
use ahash::HashMap;
use anyhow::Context;
use rust_decimal::Decimal;
use std::ops::Deref;
use std::rc::Rc;

/// 函数参数包装器
///
/// 提供对函数参数切片的便捷访问方法
pub struct Arguments<'a>(pub &'a [Variable]);

impl<'a> Deref for Arguments<'a> {
    type Target = [Variable];

    fn deref(&self) -> &'a Self::Target {
        &self.0
    }
}

impl<'a> Arguments<'a> {
    /// 获取可选变量（返回Option）
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Some(&Variable)` - 参数存在
    /// * `None` - 参数不存在（索引越界）
    pub fn ovar(
        &self,
        pos: usize,
    ) -> Option<&'a Variable> {
        self.0.get(pos)
    }

    /// 获取必需变量（返回Result）
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(&Variable)` - 参数存在
    /// * `Err` - 参数不存在或索引越界
    pub fn var(
        &self,
        pos: usize,
    ) -> anyhow::Result<&'a Variable> {
        self.ovar(pos).with_context(|| format!("参数索引越界: {pos}"))
    }

    /// 获取可选布尔值
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Some(bool))` - 参数存在且为布尔类型
    /// * `Ok(None)` - 参数不存在
    /// * `Err` - 参数存在但不是布尔类型
    pub fn obool(
        &self,
        pos: usize,
    ) -> anyhow::Result<Option<bool>> {
        match self.ovar(pos) {
            Some(v) => v
                .as_bool()
                .map(Some)
                .with_context(|| format!("参数索引 {pos} 不是布尔类型")),
            None => Ok(None),
        }
    }

    /// 获取必需布尔值
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(bool)` - 参数存在且为布尔类型
    /// * `Err` - 参数不存在或不是布尔类型
    pub fn bool(
        &self,
        pos: usize,
    ) -> anyhow::Result<bool> {
        self.obool(pos)?.with_context(|| format!("参数索引 {pos} 不是布尔类型"))
    }

    /// 获取可选字符串
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Some(&str))` - 参数存在且为字符串类型
    /// * `Ok(None)` - 参数不存在
    /// * `Err` - 参数存在但不是字符串类型
    pub fn ostr(
        &self,
        pos: usize,
    ) -> anyhow::Result<Option<&'a str>> {
        match self.ovar(pos) {
            Some(v) => v
                .as_str()
                .map(Some)
                .with_context(|| format!("参数索引 {pos} 不是字符串类型")),
            None => Ok(None),
        }
    }

    /// 获取必需字符串
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(&str)` - 参数存在且为字符串类型
    /// * `Err` - 参数不存在或不是字符串类型
    pub fn str(
        &self,
        pos: usize,
    ) -> anyhow::Result<&'a str> {
        self.ostr(pos)?
            .with_context(|| format!("参数索引 {pos} 不是字符串类型"))
    }

    /// 获取可选数字
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Some(Decimal))` - 参数存在且为数字类型
    /// * `Ok(None)` - 参数不存在
    /// * `Err` - 参数存在但不是数字类型
    pub fn onumber(
        &self,
        pos: usize,
    ) -> anyhow::Result<Option<Decimal>> {
        match self.ovar(pos) {
            Some(v) => v
                .as_number()
                .map(Some)
                .with_context(|| format!("参数索引 {pos} 不是数字类型")),
            None => Ok(None),
        }
    }

    /// 获取必需数字
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Decimal)` - 参数存在且为数字类型
    /// * `Err` - 参数不存在或不是数字类型
    pub fn number(
        &self,
        pos: usize,
    ) -> anyhow::Result<Decimal> {
        self.onumber(pos)?
            .with_context(|| format!("参数索引 {pos} 不是数字类型"))
    }

    /// 获取可选数组
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Some(RcCell<Vec<Variable>>))` - 参数存在且为数组类型
    /// * `Ok(None)` - 参数不存在
    /// * `Err` - 参数存在但不是数组类型
    pub fn oarray(
        &self,
        pos: usize,
    ) -> anyhow::Result<Option<RcCell<Vec<Variable>>>> {
        match self.ovar(pos) {
            Some(v) => v
                .as_array()
                .map(Some)
                .with_context(|| format!("参数索引 {pos} 不是数组类型")),
            None => Ok(None),
        }
    }

    /// 获取必需数组
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(RcCell<Vec<Variable>>)` - 参数存在且为数组类型
    /// * `Err` - 参数不存在或不是数组类型
    pub fn array(
        &self,
        pos: usize,
    ) -> anyhow::Result<RcCell<Vec<Variable>>> {
        self.oarray(pos)?
            .with_context(|| format!("参数索引 {pos} 不是数组类型"))
    }

    /// 获取可选对象
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Some(RcCell<HashMap<Rc<str>, Variable>>))` - 参数存在且为对象类型
    /// * `Ok(None)` - 参数不存在
    /// * `Err` - 参数存在但不是对象类型
    pub fn oobject(
        &self,
        pos: usize,
    ) -> anyhow::Result<Option<RcCell<HashMap<Rc<str>, Variable>>>> {
        match self.ovar(pos) {
            Some(v) => v
                .as_object()
                .map(Some)
                .with_context(|| format!("参数索引 {pos} 不是对象类型")),
            None => Ok(None),
        }
    }

    /// 获取必需对象
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(RcCell<HashMap<Rc<str>, Variable>>)` - 参数存在且为对象类型
    /// * `Err` - 参数不存在或不是对象类型
    pub fn object(
        &self,
        pos: usize,
    ) -> anyhow::Result<RcCell<HashMap<Rc<str>, Variable>>> {
        self.oobject(pos)?
            .with_context(|| format!("参数索引 {pos} 不是对象类型"))
    }

    /// 获取可选动态类型变量
    ///
    /// # 类型参数
    /// * `T` - 实现DynamicVariable特征的类型
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(Some(&T))` - 参数存在且为指定动态类型
    /// * `Ok(None)` - 参数不存在或不是指定动态类型
    /// * `Err` - 发生其他错误
    pub fn odynamic<T: DynamicVariable + 'static>(
        &self,
        pos: usize,
    ) -> anyhow::Result<Option<&T>> {
        match self.ovar(pos) {
            None => Ok(None),
            Some(s) => Ok(s.dynamic::<T>()),
        }
    }

    /// 获取必需动态类型变量
    ///
    /// # 类型参数
    /// * `T` - 实现DynamicVariable特征的类型
    ///
    /// # 参数
    /// * `pos` - 参数位置索引
    ///
    /// # 返回值
    /// * `Ok(&T)` - 参数存在且为指定动态类型
    /// * `Err` - 参数不存在或不是指定动态类型
    pub fn dynamic<T: DynamicVariable + 'static>(
        &self,
        pos: usize,
    ) -> anyhow::Result<&T> {
        self.odynamic(pos)?
            .with_context(|| format!("参数索引 {pos} 不是动态类型"))
    }
}
